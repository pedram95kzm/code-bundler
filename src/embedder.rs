use std::collections::{HashMap, HashSet};
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use crate::extractor::{
    self, BINARY_MARKER, FORMAT_MARKER, LENGTH_PREFIX, METADATA_SUFFIX, PREVIOUS_FORMAT_MARKER,
};

pub(crate) struct EmbedReport {
    pub(crate) output_root: PathBuf,
    pub(crate) files_created: usize,
    pub(crate) binary_files_skipped: usize,
    pub(crate) legacy_format: bool,
    pub(crate) modifications_applied: usize,
    pub(crate) files_modified: usize,
}

struct ParsedDocument {
    files: Vec<FileEntry>,
    binary_files_skipped: usize,
    legacy_format: bool,
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
    let bytes = fs::read(input_path)
        .map_err(|error| format!("cannot read '{}': {error}", input_path.display()))?;
    let document = extractor::decode_text(&bytes);
    let mut parsed = parse_document(&document)?;
    let (modifications_applied, files_modified) = match modification_path {
        Some(path) => {
            let modifications = read_modifications(path)?;
            apply_modifications(&mut parsed, modifications)?
        }
        None => (0, 0),
    };
    let output_root = create_unique_embed_root(input_path)?;

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
        legacy_format: parsed.legacy_format,
        modifications_applied,
        files_modified,
    })
}

fn read_modifications(path: &Path) -> Result<Vec<Modification>, String> {
    let bytes = fs::read(path).map_err(|error| {
        format!(
            "cannot read modification file '{}': {error}",
            path.display()
        )
    })?;
    let document = extractor::decode_text(&bytes);
    parse_modifications(&document)
        .map_err(|error| format!("invalid modification file '{}': {error}", path.display()))
}

fn parse_modifications(document: &str) -> Result<Vec<Modification>, String> {
    let mut modifications = Vec::new();
    let mut current: Option<Modification> = None;

    for (index, raw_line) in document.lines().enumerate() {
        let instruction_line = index + 1;
        let line = raw_line.strip_suffix('\r').unwrap_or(raw_line);

        match parse_modification_header(line, instruction_line) {
            Ok(Some(next)) => {
                if let Some(previous) = current.replace(next) {
                    modifications.push(previous);
                }
            }
            Ok(None) => {
                if let Some(modification) = &mut current {
                    modification.new_content.push('\n');
                    modification.new_content.push_str(line);
                } else if !line.trim().is_empty() && !line.trim_start().starts_with('#') {
                    return Err(format!("line {instruction_line} must start with 'file:'"));
                }
            }
            Err(error) => return Err(error),
        }
    }

    if let Some(last) = current {
        modifications.push(last);
    }
    if modifications.is_empty() {
        return Err("no modification records were found".to_string());
    }

    Ok(modifications)
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
    let (relative_path, path_key) = safe_relative_path(raw_path)
        .map_err(|error| format!("line {instruction_line}: {error}"))?;

    Ok(Some(Modification {
        relative_path,
        path_key,
        line_number,
        new_content: new_content.to_string(),
        instruction_line,
    }))
}

fn apply_modifications(
    parsed: &mut ParsedDocument,
    modifications: Vec<Modification>,
) -> Result<(usize, usize), String> {
    let modification_count = modifications.len();
    let entry_indices = parsed
        .files
        .iter()
        .enumerate()
        .map(|(index, entry)| {
            let (_, key) = safe_relative_path(&entry.relative_path.to_string_lossy())?;
            Ok((key, index))
        })
        .collect::<Result<HashMap<_, _>, String>>()?;

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
    let first_line = document
        .lines()
        .next()
        .unwrap_or_default()
        .trim_end_matches('\r');

    let parsed = if matches!(first_line, FORMAT_MARKER | PREVIOUS_FORMAT_MARKER) {
        parse_v2(document)?
    } else {
        parse_legacy(document)?
    };
    validate_entries(parsed)
}

fn parse_v2(document: &str) -> Result<ParsedDocument, String> {
    let bytes = document.as_bytes();
    let mut cursor = 0usize;
    let marker = take_line(document, &mut cursor)?;
    if !matches!(marker, FORMAT_MARKER | PREVIOUS_FORMAT_MARKER) {
        return Err("this is not a supported extracted text file".to_string());
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
            relative_path: safe_relative_path(raw_path)?.0,
            contents: bytes[cursor..end].to_vec(),
        });
        cursor = end;
        consume_entry_separator(bytes, &mut cursor, raw_path)?;
    }

    Ok(ParsedDocument {
        files,
        binary_files_skipped,
        legacy_format: false,
    })
}

fn parse_legacy(document: &str) -> Result<ParsedDocument, String> {
    struct Header<'a> {
        start: usize,
        content_start: usize,
        raw_path: &'a str,
    }

    let mut headers = Vec::new();
    let mut offset = 0usize;
    for line_with_ending in document.split_inclusive('\n') {
        let line = line_with_ending.trim_end_matches(['\r', '\n']);
        if let Some(raw_path) = parse_header(line) {
            headers.push(Header {
                start: offset,
                content_start: offset + line_with_ending.len(),
                raw_path,
            });
        }
        offset += line_with_ending.len();
    }

    if headers.is_empty() {
        return Err("no file headers were found in the selected text file".to_string());
    }
    if !document[..headers[0].start].trim().is_empty() {
        return Err("unexpected text appears before the first file header".to_string());
    }

    let mut files = Vec::new();
    let mut binary_files_skipped = 0usize;
    for (index, header) in headers.iter().enumerate() {
        let content_end = headers
            .get(index + 1)
            .map_or(document.len(), |next| next.start);
        let framed_contents = &document[header.content_start..content_end];
        let contents = framed_contents
            .strip_suffix('\n')
            .unwrap_or(framed_contents);

        if contents.trim_end_matches(['\r', '\n']) == BINARY_MARKER {
            binary_files_skipped += 1;
            continue;
        }

        // Version 1 had no length metadata, so it cannot distinguish an empty
        // file from a file containing only one newline. Prefer an empty file.
        let contents = if contents == "\n" { "" } else { contents };
        files.push(FileEntry {
            relative_path: safe_relative_path(header.raw_path)?.0,
            contents: contents.as_bytes().to_vec(),
        });
    }

    Ok(ParsedDocument {
        files,
        binary_files_skipped,
        legacy_format: true,
    })
}

fn validate_entries(parsed: ParsedDocument) -> Result<ParsedDocument, String> {
    let mut keys = Vec::with_capacity(parsed.files.len());
    let mut seen = HashSet::with_capacity(parsed.files.len());

    for entry in &parsed.files {
        let (_, key) = safe_relative_path(&entry.relative_path.to_string_lossy())?;
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

fn safe_relative_path(raw_path: &str) -> Result<(PathBuf, String), String> {
    if raw_path.is_empty() {
        return Err("a file header contains an empty path".to_string());
    }

    let normalized = raw_path.replace('\\', "/");
    let mut path = PathBuf::new();
    let mut key_parts = Vec::new();
    for part in normalized.split('/') {
        if part.is_empty()
            || matches!(part, "." | "..")
            || part.contains(':')
            || part.contains('\0')
        {
            return Err(format!("unsafe file path in extracted text: '{raw_path}'"));
        }
        path.push(part);
        key_parts.push(part.to_lowercase());
    }

    Ok((path, key_parts.join("/")))
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

fn create_unique_embed_root(input_path: &Path) -> Result<PathBuf, String> {
    let parent = input_path.parent().unwrap_or_else(|| Path::new("."));
    let stem = input_path
        .file_stem()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty())
        .unwrap_or("extracted_contents");

    for number in 1u32.. {
        let name = if number == 1 {
            format!("{stem}_embedded")
        } else {
            format!("{stem}_embedded_{number}")
        };
        let path = parent.join(name);
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
    fn version_two_preserves_exact_contents_and_header_like_lines() {
        let contents = "first\n==not/a/real/file content==\nlast";
        let document = format!(
            "{FORMAT_MARKER}\n==src/main.txt content==\n{LENGTH_PREFIX}{}{METADATA_SUFFIX}\n{contents}\n",
            contents.len()
        );
        let parsed = parse_document(&document).unwrap();

        assert!(!parsed.legacy_format);
        assert_eq!(parsed.files.len(), 1);
        assert_eq!(parsed.files[0].relative_path, Path::new("src/main.txt"));
        assert_eq!(parsed.files[0].contents, contents.as_bytes());
    }

    #[test]
    fn accepts_extracts_created_under_the_previous_package_name() {
        let document = format!(
            "{PREVIOUS_FORMAT_MARKER}\n==old.txt content==\n{LENGTH_PREFIX}3{METADATA_SUFFIX}\nold\n"
        );
        let parsed = parse_document(&document).unwrap();

        assert!(!parsed.legacy_format);
        assert_eq!(parsed.files[0].contents, b"old");
    }

    #[test]
    fn parses_original_legacy_format() {
        let document = "==one.txt content==\nhello\n\n==src/two.txt content==\nworld\n\n";
        let parsed = parse_document(document).unwrap();

        assert!(parsed.legacy_format);
        assert_eq!(parsed.files.len(), 2);
        assert_eq!(parsed.files[0].contents, b"hello\n");
        assert_eq!(parsed.files[1].contents, b"world\n");
    }

    #[test]
    fn rejects_paths_that_can_escape_the_output_folder() {
        let document = "==../outside.txt content==\nnope\n\n";
        let error = parse_document(document).err().unwrap();
        assert!(error.contains("unsafe file path"));
    }

    #[test]
    fn rejects_file_and_folder_path_conflicts() {
        let document = concat!(
            "==folder content==\nfile\n\n",
            "==folder/child.txt content==\nchild\n\n",
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
        let embed = embed_file(&extract.output_path, None).unwrap();

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
    fn modified_extraction_applies_changes_before_writing_files() {
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

        let bundle = crate::extractor::extract_folder(&source, false).unwrap();
        let modification_path = test_root.join("changes.txt");
        fs::write(
            &modification_path,
            concat!(
                "file:example.php,line:2,new_content:replacement A\n",
                "replacement B\n",
                "file:example.php,line:4,new_content:last replacement\n",
            ),
        )
        .unwrap();

        let report = embed_file(&bundle.output_path, Some(&modification_path)).unwrap();

        assert_eq!(report.modifications_applied, 2);
        assert_eq!(report.files_modified, 1);
        assert_eq!(
            fs::read(report.output_root.join("example.php")).unwrap(),
            b"line one\nreplacement A\nreplacement B\nline three\nlast replacement\n"
        );

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
            legacy_format: false,
        };
        let modifications = parse_modifications(concat!(
            "file:1.php,line:2,new_content:TWO-A\n",
            "TWO-B\n",
            "file:1.php,line:4,new_content:FOUR\n",
        ))
        .unwrap();

        let report = apply_modifications(&mut parsed, modifications).unwrap();

        assert_eq!(report, (2, 1));
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
            legacy_format: false,
        };
        let modifications =
            parse_modifications("file:src/file.txt,line:2,new_content:new\ncontinued").unwrap();

        apply_modifications(&mut parsed, modifications).unwrap();

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
            legacy_format: false,
        };
        let modifications = parse_modifications(concat!(
            "file:one.txt,line:1,new_content:first\n",
            "file:one.txt,line:1,new_content:second",
        ))
        .unwrap();

        let error = apply_modifications(&mut parsed, modifications).unwrap_err();

        assert!(error.contains("more than one modification"));
    }
}
