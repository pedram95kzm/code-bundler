use std::collections::{HashMap, HashSet};
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use crate::extractor::{self, BINARY_MARKER, FORMAT_MARKER, LENGTH_PREFIX, METADATA_SUFFIX};
use crate::output;
use crate::paths::{parse_portable_relative_path, portable_path_key};

pub(crate) struct EmbedReport {
    pub(crate) output_root: PathBuf,
    pub(crate) files_created: usize,
    pub(crate) binary_files_skipped: usize,
    pub(crate) modifications_applied: usize,
    pub(crate) files_modified: usize,
    pub(crate) files_added: usize,
    pub(crate) files_deleted: usize,
    pub(crate) files_renamed: usize,
}

struct ParsedDocument {
    files: Vec<FileEntry>,
    binary_files_skipped: usize,
}

struct FileEntry {
    relative_path: PathBuf,
    contents: Vec<u8>,
}

struct Modification {
    relative_path: PathBuf,
    path_key: String,
    line_number: usize,
    new_content: String,
    instruction_line: usize,
}

struct AddedFile {
    relative_path: PathBuf,
    path_key: String,
    contents: String,
    instruction_line: usize,
}

struct PathOperation {
    relative_path: PathBuf,
    path_key: String,
    instruction_line: usize,
}

struct RenameOperation {
    source_path: PathBuf,
    source_key: String,
    destination_path: PathBuf,
    instruction_line: usize,
}

#[derive(Default)]
struct ChangeSet {
    modifications: Vec<Modification>,
    additions: Vec<AddedFile>,
    deletions: Vec<PathOperation>,
    renames: Vec<RenameOperation>,
}

enum PendingContent {
    Modification(Modification),
    Addition(AddedFile),
}

enum ChangeHeader {
    Content(PendingContent),
    Deletion(PathOperation),
    Rename(RenameOperation),
}

#[derive(Default)]
struct ChangeReport {
    modifications_applied: usize,
    files_modified: usize,
    files_added: usize,
    files_deleted: usize,
    files_renamed: usize,
}

#[derive(Clone, Copy)]
struct LineSpan {
    content_start: usize,
    content_end: usize,
    ending_start: usize,
    ending_end: usize,
}

pub(crate) fn embed_file(
    input_path: &Path,
    modification_path: Option<&Path>,
) -> Result<EmbedReport, String> {
    let output_directory = output::directory()?;
    embed_file_to(input_path, modification_path, &output_directory)
}

fn embed_file_to(
    input_path: &Path,
    modification_path: Option<&Path>,
    output_directory: &Path,
) -> Result<EmbedReport, String> {
    let bytes = fs::read(input_path)
        .map_err(|error| format!("cannot read '{}': {error}", input_path.display()))?;
    let document = extractor::decode_text(&bytes).ok_or_else(|| {
        format!(
            "'{}' is not a valid UTF-8 or UTF-16 text bundle",
            input_path.display()
        )
    })?;
    let mut parsed = parse_document(&document)?;
    let change_report = match modification_path {
        Some(path) => {
            let changes = read_changes(path)?;
            apply_changes(&mut parsed, changes)?
        }
        None => ChangeReport::default(),
    };
    let output_root = create_unique_embed_root(input_path, output_directory)?;

    for entry in &parsed.files {
        let output_path = output_root.join(&entry.relative_path);
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                embed_error(&output_root, &output_path, "create its folder", error)
            })?;
        }

        let mut output = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&output_path)
            .map_err(|error| embed_error(&output_root, &output_path, "create it", error))?;
        output
            .write_all(&entry.contents)
            .map_err(|error| embed_error(&output_root, &output_path, "write it", error))?;
    }

    Ok(EmbedReport {
        output_root,
        files_created: parsed.files.len(),
        binary_files_skipped: parsed.binary_files_skipped,
        modifications_applied: change_report.modifications_applied,
        files_modified: change_report.files_modified,
        files_added: change_report.files_added,
        files_deleted: change_report.files_deleted,
        files_renamed: change_report.files_renamed,
    })
}

fn read_changes(path: &Path) -> Result<ChangeSet, String> {
    let bytes = fs::read(path).map_err(|error| {
        format!(
            "cannot read modification file '{}': {error}",
            path.display()
        )
    })?;
    let document = extractor::decode_text(&bytes).ok_or_else(|| {
        format!(
            "modification file '{}' is not valid UTF-8 or UTF-16 text",
            path.display()
        )
    })?;
    parse_changes(&document)
        .map_err(|error| format!("invalid modification file '{}': {error}", path.display()))
}

fn parse_changes(document: &str) -> Result<ChangeSet, String> {
    let mut changes = ChangeSet::default();
    let mut current: Option<PendingContent> = None;

    for (index, raw_line) in document.lines().enumerate() {
        let instruction_line = index + 1;
        let line = raw_line.strip_suffix('\r').unwrap_or(raw_line);

        if let Some(content) = &mut current
            && let Some(escaped_line) = line.strip_prefix('\\')
        {
            append_content_line(content, escaped_line);
            continue;
        }

        match parse_change_header(line, instruction_line)? {
            Some(ChangeHeader::Content(next)) => {
                if let Some(previous) = current.replace(next) {
                    push_pending_content(&mut changes, previous);
                }
            }
            Some(ChangeHeader::Deletion(deletion)) => {
                if let Some(previous) = current.take() {
                    push_pending_content(&mut changes, previous);
                }
                changes.deletions.push(deletion);
            }
            Some(ChangeHeader::Rename(rename)) => {
                if let Some(previous) = current.take() {
                    push_pending_content(&mut changes, previous);
                }
                changes.renames.push(rename);
            }
            None => {
                if let Some(content) = &mut current {
                    append_content_line(content, line);
                } else if !line.trim().is_empty() && !line.trim_start().starts_with('#') {
                    return Err(format!(
                        "line {instruction_line} must start with 'file:', 'add_file:', 'delete_file:', or 'rename_file:'"
                    ));
                }
            }
        }
    }

    if let Some(last) = current {
        push_pending_content(&mut changes, last);
    }
    if changes.modifications.is_empty()
        && changes.additions.is_empty()
        && changes.deletions.is_empty()
        && changes.renames.is_empty()
    {
        return Err("no change records were found".to_string());
    }

    Ok(changes)
}

fn parse_change_header(
    line: &str,
    instruction_line: usize,
) -> Result<Option<ChangeHeader>, String> {
    if line.starts_with("file:") {
        return parse_modification_header(line, instruction_line).map(|record| {
            record.map(|value| ChangeHeader::Content(PendingContent::Modification(value)))
        });
    }
    if line.starts_with("add_file:") {
        return parse_addition_header(line, instruction_line)
            .map(|value| Some(ChangeHeader::Content(PendingContent::Addition(value))));
    }
    if line.starts_with("delete_file:") {
        return parse_deletion_header(line, instruction_line)
            .map(|value| Some(ChangeHeader::Deletion(value)));
    }
    if line.starts_with("rename_file:") {
        return parse_rename_header(line, instruction_line)
            .map(|value| Some(ChangeHeader::Rename(value)));
    }
    Ok(None)
}

fn push_pending_content(changes: &mut ChangeSet, content: PendingContent) {
    match content {
        PendingContent::Modification(modification) => changes.modifications.push(modification),
        PendingContent::Addition(addition) => changes.additions.push(addition),
    }
}

fn append_content_line(content: &mut PendingContent, line: &str) {
    let value = match content {
        PendingContent::Modification(modification) => &mut modification.new_content,
        PendingContent::Addition(addition) => &mut addition.contents,
    };
    value.push('\n');
    value.push_str(line);
}

fn parse_modification_header(
    line: &str,
    instruction_line: usize,
) -> Result<Option<Modification>, String> {
    let Some(record) = line.strip_prefix("file:") else {
        return Ok(None);
    };
    let Some((raw_path, remainder)) = record.split_once(",line:") else {
        // A multiline replacement may itself contain a line beginning with
        // "file:". It is only a record when it includes the record delimiters.
        if record.contains(",new_content:") {
            return Err(format!(
                "line {instruction_line} is missing the ',line:' field"
            ));
        }
        return Ok(None);
    };
    let Some((raw_line_number, new_content)) = remainder.split_once(",new_content:") else {
        return Err(format!(
            "line {instruction_line} is missing the ',new_content:' field"
        ));
    };
    if raw_path.is_empty() {
        return Err(format!("line {instruction_line} has an empty file path"));
    }
    let line_number = raw_line_number.parse::<usize>().map_err(|_| {
        format!("line {instruction_line} has an invalid target line number '{raw_line_number}'")
    })?;
    if line_number == 0 {
        return Err(format!(
            "line {instruction_line} has target line 0; line numbers start at 1"
        ));
    }
    let (relative_path, path_key) = parse_portable_relative_path(raw_path)
        .map_err(|error| format!("line {instruction_line}: {error}"))?;

    Ok(Some(Modification {
        relative_path,
        path_key,
        line_number,
        new_content: new_content.to_string(),
        instruction_line,
    }))
}

fn parse_addition_header(line: &str, instruction_line: usize) -> Result<AddedFile, String> {
    let record = line
        .strip_prefix("add_file:")
        .expect("addition prefix was checked");
    let Some((raw_path, contents)) = record.split_once(",new_content:") else {
        return Err(format!(
            "line {instruction_line} is missing the ',new_content:' field"
        ));
    };
    let (relative_path, path_key) = parse_portable_relative_path(raw_path)
        .map_err(|error| format!("line {instruction_line}: {error}"))?;
    Ok(AddedFile {
        relative_path,
        path_key,
        contents: contents.to_owned(),
        instruction_line,
    })
}

fn parse_deletion_header(line: &str, instruction_line: usize) -> Result<PathOperation, String> {
    let raw_path = line
        .strip_prefix("delete_file:")
        .expect("deletion prefix was checked");
    let (relative_path, path_key) = parse_portable_relative_path(raw_path)
        .map_err(|error| format!("line {instruction_line}: {error}"))?;
    Ok(PathOperation {
        relative_path,
        path_key,
        instruction_line,
    })
}

fn parse_rename_header(line: &str, instruction_line: usize) -> Result<RenameOperation, String> {
    let record = line
        .strip_prefix("rename_file:")
        .expect("rename prefix was checked");
    let Some((raw_source, raw_destination)) = record.split_once(",new_path:") else {
        return Err(format!(
            "line {instruction_line} is missing the ',new_path:' field"
        ));
    };
    let (source_path, source_key) = parse_portable_relative_path(raw_source)
        .map_err(|error| format!("line {instruction_line}: {error}"))?;
    let (destination_path, _) = parse_portable_relative_path(raw_destination)
        .map_err(|error| format!("line {instruction_line}: {error}"))?;
    if source_path == destination_path {
        return Err(format!(
            "line {instruction_line} renames '{}' to the same path",
            source_path.display()
        ));
    }
    Ok(RenameOperation {
        source_path,
        source_key,
        destination_path,
        instruction_line,
    })
}

fn apply_changes(parsed: &mut ParsedDocument, changes: ChangeSet) -> Result<ChangeReport, String> {
    let ChangeSet {
        modifications,
        additions,
        deletions,
        renames,
    } = changes;
    let entry_indices = parsed
        .files
        .iter()
        .enumerate()
        .map(|(index, entry)| Ok((portable_path_key(&entry.relative_path)?, index)))
        .collect::<Result<HashMap<_, _>, String>>()?;

    let mut source_operations = HashMap::new();
    for deletion in &deletions {
        if !entry_indices.contains_key(&deletion.path_key) {
            return Err(format!(
                "change line {} refers to '{}', which is not a stored text file in the bundle",
                deletion.instruction_line,
                deletion.relative_path.display()
            ));
        }
        if let Some(previous_line) =
            source_operations.insert(deletion.path_key.clone(), deletion.instruction_line)
        {
            return Err(format!(
                "change lines {previous_line} and {} both operate on '{}'",
                deletion.instruction_line,
                deletion.relative_path.display()
            ));
        }
    }
    for rename in &renames {
        if !entry_indices.contains_key(&rename.source_key) {
            return Err(format!(
                "change line {} refers to '{}', which is not a stored text file in the bundle",
                rename.instruction_line,
                rename.source_path.display()
            ));
        }
        if let Some(previous_line) =
            source_operations.insert(rename.source_key.clone(), rename.instruction_line)
        {
            return Err(format!(
                "change lines {previous_line} and {} both operate on '{}'",
                rename.instruction_line,
                rename.source_path.display()
            ));
        }
    }

    let mut addition_targets = HashMap::new();
    for addition in &additions {
        if let Some(previous_line) =
            addition_targets.insert(addition.path_key.clone(), addition.instruction_line)
        {
            return Err(format!(
                "change lines {previous_line} and {} both add '{}'",
                addition.instruction_line,
                addition.relative_path.display()
            ));
        }
    }

    let deleted_keys = deletions
        .iter()
        .map(|deletion| deletion.path_key.clone())
        .collect::<HashSet<_>>();
    for modification in &modifications {
        if deleted_keys.contains(&modification.path_key) {
            return Err(format!(
                "change line {} modifies '{}', but that file is also deleted",
                modification.instruction_line,
                modification.relative_path.display()
            ));
        }
    }

    let (modifications_applied, files_modified) =
        apply_line_modifications(parsed, modifications, &entry_indices)?;

    let rename_map = renames
        .into_iter()
        .map(|rename| (rename.source_key.clone(), rename))
        .collect::<HashMap<_, _>>();
    let mut retained_files = Vec::with_capacity(parsed.files.len());
    for entry in std::mem::take(&mut parsed.files) {
        let key = portable_path_key(&entry.relative_path)?;
        if !deleted_keys.contains(&key) {
            retained_files.push(entry);
        }
    }
    parsed.files = retained_files;
    for entry in &mut parsed.files {
        let key = portable_path_key(&entry.relative_path)?;
        if let Some(rename) = rename_map.get(&key) {
            entry.relative_path = rename.destination_path.clone();
        }
    }
    for addition in additions {
        parsed.files.push(FileEntry {
            relative_path: addition.relative_path,
            contents: addition.contents.into_bytes(),
        });
    }

    let final_document = validate_entries(ParsedDocument {
        files: std::mem::take(&mut parsed.files),
        binary_files_skipped: parsed.binary_files_skipped,
    })
    .map_err(|error| format!("the requested file changes conflict: {error}"))?;
    *parsed = final_document;

    Ok(ChangeReport {
        modifications_applied,
        files_modified,
        files_added: addition_targets.len(),
        files_deleted: deleted_keys.len(),
        files_renamed: rename_map.len(),
    })
}

fn apply_line_modifications(
    parsed: &mut ParsedDocument,
    modifications: Vec<Modification>,
    entry_indices: &HashMap<String, usize>,
) -> Result<(usize, usize), String> {
    let modification_count = modifications.len();
    let mut grouped: HashMap<usize, Vec<Modification>> = HashMap::new();
    let mut targets = HashSet::new();
    for modification in modifications {
        let Some(&entry_index) = entry_indices.get(&modification.path_key) else {
            return Err(format!(
                "modification line {} refers to '{}', which is not in the bundled text",
                modification.instruction_line,
                modification.relative_path.display()
            ));
        };
        if !targets.insert((entry_index, modification.line_number)) {
            return Err(format!(
                "more than one modification targets line {} of '{}'",
                modification.line_number,
                modification.relative_path.display()
            ));
        }
        grouped.entry(entry_index).or_default().push(modification);
    }

    let files_modified = grouped.len();
    for (entry_index, mut file_modifications) in grouped {
        let entry = &mut parsed.files[entry_index];
        let mut contents = String::from_utf8(entry.contents.clone()).map_err(|_| {
            format!(
                "'{}' is not valid UTF-8 and cannot be modified by line",
                entry.relative_path.display()
            )
        })?;
        let spans = line_spans(&contents);
        let default_ending = preferred_line_ending(&contents);

        for modification in &file_modifications {
            if modification.line_number > spans.len() {
                return Err(format!(
                    "modification line {} targets line {} of '{}', but that file has {} line(s)",
                    modification.instruction_line,
                    modification.line_number,
                    modification.relative_path.display(),
                    spans.len()
                ));
            }
        }

        // All coordinates refer to the original file. Replacing from the
        // bottom upward keeps every earlier byte range stable even when a
        // replacement adds or removes lines.
        file_modifications
            .sort_unstable_by_key(|modification| std::cmp::Reverse(modification.line_number));
        for modification in file_modifications {
            let span = spans[modification.line_number - 1];
            let ending = if span.ending_start < span.ending_end {
                &contents[span.ending_start..span.ending_end]
            } else {
                default_ending
            };
            let replacement = normalize_line_endings(&modification.new_content, ending);
            contents.replace_range(span.content_start..span.content_end, &replacement);
        }
        entry.contents = contents.into_bytes();
    }

    Ok((modification_count, files_modified))
}

fn line_spans(contents: &str) -> Vec<LineSpan> {
    let bytes = contents.as_bytes();
    let mut spans = Vec::new();
    let mut line_start = 0usize;

    for (index, byte) in bytes.iter().enumerate() {
        if *byte != b'\n' {
            continue;
        }
        let ending_start = if index > line_start && bytes[index - 1] == b'\r' {
            index - 1
        } else {
            index
        };
        spans.push(LineSpan {
            content_start: line_start,
            content_end: ending_start,
            ending_start,
            ending_end: index + 1,
        });
        line_start = index + 1;
    }

    if line_start < bytes.len() {
        spans.push(LineSpan {
            content_start: line_start,
            content_end: bytes.len(),
            ending_start: bytes.len(),
            ending_end: bytes.len(),
        });
    }

    spans
}

fn preferred_line_ending(contents: &str) -> &'static str {
    if contents.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    }
}

fn normalize_line_endings(contents: &str, line_ending: &str) -> String {
    let normalized = contents.replace("\r\n", "\n").replace('\r', "\n");
    if line_ending == "\n" {
        normalized
    } else {
        normalized.replace('\n', line_ending)
    }
}

fn parse_document(document: &str) -> Result<ParsedDocument, String> {
    let bytes = document.as_bytes();
    let mut cursor = 0usize;
    let marker = take_line(document, &mut cursor)?;
    if marker != FORMAT_MARKER {
        return Err(format!(
            "this is not a Code Bundler v1 file (expected '{FORMAT_MARKER}')"
        ));
    }

    let mut files = Vec::new();
    let mut binary_files_skipped = 0usize;

    while cursor < bytes.len() {
        let header = take_line(document, &mut cursor)?;
        if header.is_empty() {
            continue;
        }
        let raw_path = parse_header(header)
            .ok_or_else(|| format!("invalid file header at byte {cursor}: '{header}'"))?;
        let metadata = take_line(document, &mut cursor)?;

        if metadata == BINARY_MARKER {
            binary_files_skipped += 1;
            continue;
        }

        let length = metadata
            .strip_prefix(LENGTH_PREFIX)
            .and_then(|value| value.strip_suffix(METADATA_SUFFIX))
            .ok_or_else(|| format!("missing content length after header for '{raw_path}'"))?
            .parse::<usize>()
            .map_err(|_| format!("invalid content length for '{raw_path}'"))?;
        let end = cursor
            .checked_add(length)
            .filter(|end| *end <= bytes.len())
            .ok_or_else(|| format!("content for '{raw_path}' is shorter than its byte count"))?;
        if !document.is_char_boundary(end) {
            return Err(format!(
                "content length for '{raw_path}' ends inside a UTF-8 character"
            ));
        }

        files.push(FileEntry {
            relative_path: parse_portable_relative_path(raw_path)?.0,
            contents: bytes[cursor..end].to_vec(),
        });
        cursor = end;
        consume_entry_separator(bytes, &mut cursor, raw_path)?;
    }

    validate_entries(ParsedDocument {
        files,
        binary_files_skipped,
    })
}

fn validate_entries(parsed: ParsedDocument) -> Result<ParsedDocument, String> {
    let mut keys = Vec::with_capacity(parsed.files.len());
    let mut seen = HashSet::with_capacity(parsed.files.len());

    for entry in &parsed.files {
        let key = portable_path_key(&entry.relative_path)?;
        if !seen.insert(key.clone()) {
            return Err(format!(
                "the extracted text contains the file path '{}' more than once",
                entry.relative_path.display()
            ));
        }
        keys.push(key);
    }

    keys.sort_unstable();
    for pair in keys.windows(2) {
        if pair[1].starts_with(&(pair[0].clone() + "/")) {
            return Err(format!(
                "'{}' cannot be both a file and a parent folder",
                pair[0]
            ));
        }
    }

    Ok(parsed)
}

fn parse_header(line: &str) -> Option<&str> {
    line.strip_prefix("==")
        .and_then(|value| value.strip_suffix(" content=="))
        .filter(|path| !path.is_empty())
}

fn take_line<'a>(document: &'a str, cursor: &mut usize) -> Result<&'a str, String> {
    if *cursor >= document.len() {
        return Err("the extracted text ends unexpectedly".to_string());
    }

    let remainder = &document[*cursor..];
    let (line, consumed) = match remainder.find('\n') {
        Some(index) => (&remainder[..index], index + 1),
        None => (remainder, remainder.len()),
    };
    *cursor += consumed;
    Ok(line.strip_suffix('\r').unwrap_or(line))
}

fn consume_entry_separator(bytes: &[u8], cursor: &mut usize, raw_path: &str) -> Result<(), String> {
    if *cursor == bytes.len() {
        return Ok(());
    }
    if bytes[*cursor] == b'\n' {
        *cursor += 1;
        return Ok(());
    }
    if bytes.get(*cursor..*cursor + 2) == Some(b"\r\n") {
        *cursor += 2;
        return Ok(());
    }
    Err(format!(
        "content for '{raw_path}' is not followed by an entry separator"
    ))
}

fn create_unique_embed_root(input_path: &Path, output_directory: &Path) -> Result<PathBuf, String> {
    let stem = input_path
        .file_stem()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty())
        .unwrap_or("bundle");

    fs::create_dir_all(output_directory).map_err(|error| {
        format!(
            "cannot create output folder '{}': {error}",
            output_directory.display()
        )
    })?;

    for number in 1u32.. {
        let name = if number == 1 {
            format!("{stem}_embedded")
        } else {
            format!("{stem}_embedded_{number}")
        };
        let path = output_directory.join(name);
        match fs::create_dir(&path) {
            Ok(()) => return Ok(path),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(format!(
                    "cannot create embed folder '{}': {error}",
                    path.display()
                ));
            }
        }
    }
    unreachable!("the embed folder counter cannot be exhausted")
}

fn embed_error(root: &Path, path: &Path, action: &str, error: io::Error) -> String {
    format!(
        "could not {action} '{}': {error}. The incomplete embed is in '{}'",
        path.display(),
        root.display()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn current_format_preserves_exact_contents_and_header_like_lines() {
        let contents = "first\n==not/a/real/file content==\nlast";
        let document = format!(
            "{FORMAT_MARKER}\n==src/main.txt content==\n{LENGTH_PREFIX}{}{METADATA_SUFFIX}\n{contents}\n",
            contents.len()
        );
        let parsed = parse_document(&document).unwrap();

        assert_eq!(parsed.files.len(), 1);
        assert_eq!(parsed.files[0].relative_path, Path::new("src/main.txt"));
        assert_eq!(parsed.files[0].contents, contents.as_bytes());
    }

    #[test]
    fn rejects_unversioned_formats() {
        let document = "==one.txt content==\nhello\n\n==src/two.txt content==\nworld\n\n";
        let error = parse_document(document).err().unwrap();

        assert!(error.contains("expected '==code-bundler:v1=='"));
    }

    #[test]
    fn rejects_paths_that_can_escape_the_output_folder() {
        let document = format!(
            "{FORMAT_MARKER}\n==../outside.txt content==\n{LENGTH_PREFIX}4{METADATA_SUFFIX}\nnope\n"
        );
        let error = parse_document(&document).err().unwrap();
        assert!(error.contains("unsafe or non-portable file path"));
    }

    #[test]
    fn rejects_file_and_folder_path_conflicts() {
        let document = concat!(
            "==code-bundler:v1==\n",
            "==folder content==\n==utf8-bytes:4==\nfile\n",
            "==folder/child.txt content==\n==utf8-bytes:5==\nchild\n",
        );
        let error = parse_document(document).err().unwrap();
        assert!(error.contains("both a file and a parent folder"));
    }

    #[test]
    fn extract_then_embed_round_trip_recreates_files_and_folders() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let test_root = std::env::temp_dir().join(format!(
            "code-bundler-round-trip-{}-{unique}",
            std::process::id()
        ));
        let source = test_root.join("project");
        fs::create_dir_all(source.join("src")).unwrap();
        fs::write(source.join("empty.txt"), b"").unwrap();
        fs::write(source.join("no-final-newline.txt"), b"exact").unwrap();
        fs::write(
            source.join("src").join("header-like.txt"),
            b"before\n==fake.txt content==\nafter\n",
        )
        .unwrap();

        let extract = crate::extractor::extract_folder(&source, false).unwrap();
        let embed = embed_file_to(&extract.output_path, None, &test_root.join("output")).unwrap();

        assert_eq!(fs::read(embed.output_root.join("empty.txt")).unwrap(), b"");
        assert_eq!(
            fs::read(embed.output_root.join("no-final-newline.txt")).unwrap(),
            b"exact"
        );
        assert_eq!(
            fs::read(embed.output_root.join("src").join("header-like.txt")).unwrap(),
            b"before\n==fake.txt content==\nafter\n"
        );

        fs::remove_dir_all(test_root).unwrap();
    }

    #[test]
    fn changed_extraction_applies_content_and_file_operations_before_writing() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let test_root = std::env::temp_dir().join(format!(
            "code-bundler-modified-extract-{}-{unique}",
            std::process::id()
        ));
        let source = test_root.join("project");
        fs::create_dir_all(&source).unwrap();
        fs::write(
            source.join("example.php"),
            b"line one\nline two\nline three\nline four\n",
        )
        .unwrap();
        fs::write(source.join("obsolete.txt"), b"remove me\n").unwrap();

        let bundle = crate::extractor::extract_folder(&source, false).unwrap();
        let modification_path = test_root.join("changes.txt");
        fs::write(
            &modification_path,
            concat!(
                "file:example.php,line:2,new_content:replacement A\n",
                "replacement B\n",
                "file:example.php,line:4,new_content:last replacement\n",
                "rename_file:example.php,new_path:src/example.php\n",
                "delete_file:obsolete.txt\n",
                "add_file:src/new.txt,new_content:first new line\n",
                "second new line",
            ),
        )
        .unwrap();

        let output_directory = test_root.join("Documents").join("code_bundler");
        let report = embed_file_to(
            &bundle.output_path,
            Some(&modification_path),
            &output_directory,
        )
        .unwrap();

        assert_eq!(report.output_root.parent(), Some(output_directory.as_path()));

        assert_eq!(report.modifications_applied, 2);
        assert_eq!(report.files_modified, 1);
        assert_eq!(report.files_added, 1);
        assert_eq!(report.files_deleted, 1);
        assert_eq!(report.files_renamed, 1);
        assert_eq!(report.files_created, 2);
        assert_eq!(
            fs::read(report.output_root.join("src").join("example.php")).unwrap(),
            b"line one\nreplacement A\nreplacement B\nline three\nlast replacement\n"
        );
        assert_eq!(
            fs::read(report.output_root.join("src").join("new.txt")).unwrap(),
            b"first new line\nsecond new line"
        );
        assert!(!report.output_root.join("example.php").exists());
        assert!(!report.output_root.join("obsolete.txt").exists());

        fs::remove_dir_all(test_root).unwrap();
    }

    #[test]
    fn multiline_modifications_use_original_line_numbers() {
        let mut parsed = ParsedDocument {
            files: vec![FileEntry {
                relative_path: PathBuf::from("1.php"),
                contents: b"one\ntwo\nthree\nfour\n".to_vec(),
            }],
            binary_files_skipped: 0,
        };
        let changes = parse_changes(concat!(
            "file:1.php,line:2,new_content:TWO-A\n",
            "TWO-B\n",
            "file:1.php,line:4,new_content:FOUR\n",
        ))
        .unwrap();

        let report = apply_changes(&mut parsed, changes).unwrap();

        assert_eq!(report.modifications_applied, 2);
        assert_eq!(report.files_modified, 1);
        assert_eq!(
            parsed.files[0].contents,
            b"one\nTWO-A\nTWO-B\nthree\nFOUR\n"
        );
    }

    #[test]
    fn modifications_preserve_crlf_line_endings() {
        let mut parsed = ParsedDocument {
            files: vec![FileEntry {
                relative_path: PathBuf::from("src/file.txt"),
                contents: b"first\r\nsecond\r\nthird".to_vec(),
            }],
            binary_files_skipped: 0,
        };
        let changes = parse_changes("file:src/file.txt,line:2,new_content:new\ncontinued").unwrap();

        apply_changes(&mut parsed, changes).unwrap();

        assert_eq!(
            parsed.files[0].contents,
            b"first\r\nnew\r\ncontinued\r\nthird"
        );
    }

    #[test]
    fn duplicate_targets_are_rejected() {
        let mut parsed = ParsedDocument {
            files: vec![FileEntry {
                relative_path: PathBuf::from("one.txt"),
                contents: b"original".to_vec(),
            }],
            binary_files_skipped: 0,
        };
        let changes = parse_changes(concat!(
            "file:one.txt,line:1,new_content:first\n",
            "file:one.txt,line:1,new_content:second",
        ))
        .unwrap();

        let error = apply_changes(&mut parsed, changes).err().unwrap();

        assert!(error.contains("more than one modification"));
    }

    #[test]
    fn file_operations_must_produce_a_conflict_free_tree() {
        let mut parsed = ParsedDocument {
            files: vec![
                FileEntry {
                    relative_path: PathBuf::from("one.txt"),
                    contents: b"one".to_vec(),
                },
                FileEntry {
                    relative_path: PathBuf::from("two.txt"),
                    contents: b"two".to_vec(),
                },
            ],
            binary_files_skipped: 0,
        };
        let changes = parse_changes("rename_file:one.txt,new_path:two.txt").unwrap();

        let error = apply_changes(&mut parsed, changes).err().unwrap();

        assert!(error.contains("requested file changes conflict"));
        assert!(error.contains("more than once"));
    }

    #[test]
    fn a_deleted_file_cannot_also_be_modified() {
        let mut parsed = ParsedDocument {
            files: vec![FileEntry {
                relative_path: PathBuf::from("one.txt"),
                contents: b"original".to_vec(),
            }],
            binary_files_skipped: 0,
        };
        let changes = parse_changes(concat!(
            "file:one.txt,line:1,new_content:changed\n",
            "delete_file:one.txt",
        ))
        .unwrap();

        let error = apply_changes(&mut parsed, changes).err().unwrap();

        assert!(error.contains("also deleted"));
    }

    #[test]
    fn escaped_continuation_lines_can_look_like_records() {
        let changes = parse_changes(concat!(
            "add_file:notes.txt,new_content:first\n",
            "\\delete_file:not-an-operation\n",
            "\\\\leading backslash",
        ))
        .unwrap();

        assert_eq!(changes.additions.len(), 1);
        assert_eq!(
            changes.additions[0].contents,
            "first\ndelete_file:not-an-operation\n\\leading backslash"
        );
    }
}
