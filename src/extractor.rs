use std::collections::HashSet;
use std::fs::{self, File, OpenOptions};
use std::io::{BufWriter, Read, Write};
use std::path::{Path, PathBuf};

use ignore::WalkBuilder;

use crate::compression;
use crate::paths::portable_relative_display;

#[cfg(test)]
const OUTPUT_NAME: &str = "extracted_content_test.txt";
pub(crate) const FORMAT_MARKER: &str = "==code-bundler:v1==";
pub(crate) const LENGTH_PREFIX: &str = "==utf8-bytes:";
pub(crate) const METADATA_SUFFIX: &str = "==";
pub(crate) const BINARY_MARKER: &str = "[Skipped: binary file]";

pub(crate) struct ExtractionReport {
    pub(crate) output_path: PathBuf,
    pub(crate) text_files: usize,
    pub(crate) binary_files: usize,
    pub(crate) warnings: Vec<String>,
}

struct SourceFile {
    path: PathBuf,
    relative: String,
}

#[cfg(test)]
pub(crate) fn extract_folder(root: &Path, compress: bool) -> Result<ExtractionReport, String> {
    let (mut files, mut warnings) = collect_files(root, None)?;
    let (output_path, output_file) = create_unique_output(root)?;
    write_bundle(
        compress,
        &mut files,
        &mut warnings,
        output_path,
        output_file,
    )
}

pub(crate) fn extract_folder_to(
    root: &Path,
    output_path: &Path,
    compress: bool,
) -> Result<ExtractionReport, String> {
    let (mut files, mut warnings) = collect_files(root, output_path.parent())?;
    let output_file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output_path)
        .map_err(|error| {
            format!(
                "cannot create output file '{}': {error}",
                output_path.display()
            )
        })?;

    let result = write_bundle(
        compress,
        &mut files,
        &mut warnings,
        output_path.to_path_buf(),
        output_file,
    );
    if let Err(error) = result {
        return match fs::remove_file(output_path) {
            Ok(()) => Err(error),
            Err(cleanup_error) if cleanup_error.kind() == std::io::ErrorKind::NotFound => {
                Err(error)
            }
            Err(cleanup_error) => Err(format!(
                "{error}; also could not remove the incomplete output '{}': {cleanup_error}",
                output_path.display()
            )),
        };
    }
    result
}

fn write_bundle(
    compress: bool,
    files: &mut [SourceFile],
    warnings: &mut Vec<String>,
    output_path: PathBuf,
    output_file: File,
) -> Result<ExtractionReport, String> {
    files.sort_by(|left, right| {
        left.relative
            .to_lowercase()
            .cmp(&right.relative.to_lowercase())
    });

    let mut output = BufWriter::new(output_file);
    let mut text_files = 0usize;
    let mut binary_files = 0usize;

    writeln!(output, "{FORMAT_MARKER}").map_err(|error| output_error(&output_path, error))?;

    for source in files.iter() {
        let path = &source.path;
        let relative = &source.relative;
        let mut contents = Vec::new();
        match File::open(path).and_then(|mut file| file.read_to_end(&mut contents)) {
            Ok(_) => {
                writeln!(output, "=={relative} content==")
                    .map_err(|error| output_error(&output_path, error))?;

                if is_probably_binary(&contents) {
                    writeln!(output, "{BINARY_MARKER}\n")
                        .map_err(|error| output_error(&output_path, error))?;
                    binary_files += 1;
                } else {
                    let Some(decoded) = decode_text(&contents) else {
                        writeln!(output, "{BINARY_MARKER}\n")
                            .map_err(|error| output_error(&output_path, error))?;
                        binary_files += 1;
                        continue;
                    };
                    let text = if compress {
                        compression::safely_compress(path, &decoded)
                    } else {
                        decoded
                    };
                    writeln!(output, "{LENGTH_PREFIX}{}{METADATA_SUFFIX}", text.len())
                        .map_err(|error| output_error(&output_path, error))?;
                    write!(output, "{text}").map_err(|error| output_error(&output_path, error))?;
                    // This newline separates entries. The byte count above lets
                    // the embedder distinguish it from the file's contents.
                    writeln!(output).map_err(|error| output_error(&output_path, error))?;
                    text_files += 1;
                }
            }
            Err(error) => warnings.push(format!("{relative}: {error}")),
        }
    }

    output
        .flush()
        .map_err(|error| output_error(&output_path, error))?;

    Ok(ExtractionReport {
        output_path,
        text_files,
        binary_files,
        warnings: std::mem::take(warnings),
    })
}

fn collect_files(
    root: &Path,
    excluded_directory: Option<&Path>,
) -> Result<(Vec<SourceFile>, Vec<String>), String> {
    let root = fs::canonicalize(root)
        .map_err(|error| format!("cannot read folder '{}': {error}", root.display()))?;
    let excluded_directory = excluded_directory
        .and_then(|path| fs::canonicalize(path).ok())
        .filter(|path| path != &root && path.starts_with(&root));

    let mut files = Vec::new();
    let mut warnings = Vec::new();

    let mut builder = WalkBuilder::new(&root);
    builder
        .hidden(false)
        .parents(false)
        .ignore(false)
        .git_global(false)
        .git_exclude(false)
        .git_ignore(true)
        .require_git(false)
        .follow_links(false)
        .filter_entry(move |entry| {
            (entry.depth() == 0 || excluded_directory.as_deref() != Some(entry.path()))
                && (entry.depth() == 0
                    || !matches!(
                        entry.file_name().to_str(),
                        Some(".git" | ".hg" | ".svn" | ".jj")
                    ))
        });

    for result in builder.build() {
        match result {
            Ok(entry) => {
                if let Some(error) = entry.error() {
                    warnings.push(format!("{}: {error}", entry.path().display()));
                }
                if entry.file_type().is_some_and(|kind| kind.is_file())
                    && !is_generated_output(&root, entry.path())
                {
                    let path = entry.into_path();
                    let relative = portable_relative_display(&root, &path)?;
                    files.push(SourceFile { path, relative });
                }
            }
            Err(error) => warnings.push(error.to_string()),
        }
    }

    validate_source_paths(&files)?;
    Ok((files, warnings))
}

fn validate_source_paths(files: &[SourceFile]) -> Result<(), String> {
    let mut keys = files
        .iter()
        .map(|file| file.relative.to_lowercase())
        .collect::<Vec<_>>();
    let mut seen = HashSet::with_capacity(keys.len());
    for (file, key) in files.iter().zip(&keys) {
        if !seen.insert(key) {
            return Err(format!(
                "the project contains file paths that differ only by letter case; '{}' cannot be restored portably",
                file.relative
            ));
        }
    }

    keys.sort_unstable();
    for pair in keys.windows(2) {
        if pair[1].starts_with(&(pair[0].clone() + "/")) {
            return Err(format!(
                "the project path '{}' conflicts with a parent file on another platform",
                pair[0]
            ));
        }
    }
    Ok(())
}

fn is_generated_output(root: &Path, path: &Path) -> bool {
    if path.parent() != Some(root) {
        return false;
    }

    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    let Some(stem) = name.strip_suffix(".txt") else {
        return false;
    };

    stem.starts_with("extracted_content_") || stem.starts_with("generated_prompt_")
}

#[cfg(test)]
fn create_unique_output(root: &Path) -> Result<(PathBuf, File), String> {
    for number in 1u32.. {
        let name = if number == 1 {
            OUTPUT_NAME.to_string()
        } else {
            format!("extracted_content_test_{number}.txt")
        };
        let path = root.join(name);
        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(file) => return Ok((path, file)),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(format!(
                    "cannot create output file '{}': {error}",
                    path.display()
                ));
            }
        }
    }
    unreachable!("the output file counter cannot be exhausted")
}

fn is_probably_binary(bytes: &[u8]) -> bool {
    bytes.iter().take(8_192).any(|byte| *byte == 0)
        && !bytes.starts_with(&[0xff, 0xfe])
        && !bytes.starts_with(&[0xfe, 0xff])
}

pub(crate) fn decode_text(bytes: &[u8]) -> Option<String> {
    if bytes.starts_with(&[0xff, 0xfe, 0x00, 0x00]) || bytes.starts_with(&[0x00, 0x00, 0xfe, 0xff])
    {
        return None;
    }
    if bytes.starts_with(&[0xff, 0xfe]) {
        if !bytes.len().is_multiple_of(2) {
            return None;
        }
        let units = bytes[2..]
            .chunks_exact(2)
            .map(|pair| u16::from_le_bytes([pair[0], pair[1]]));
        return char::decode_utf16(units)
            .collect::<Result<String, _>>()
            .ok();
    }
    if bytes.starts_with(&[0xfe, 0xff]) {
        if !bytes.len().is_multiple_of(2) {
            return None;
        }
        let units = bytes[2..]
            .chunks_exact(2)
            .map(|pair| u16::from_be_bytes([pair[0], pair[1]]));
        return char::decode_utf16(units)
            .collect::<Result<String, _>>()
            .ok();
    }

    std::str::from_utf8(bytes.strip_prefix(&[0xef, 0xbb, 0xbf]).unwrap_or(bytes))
        .ok()
        .map(str::to_owned)
}

fn output_error(path: &Path, error: std::io::Error) -> String {
    format!("could not write '{}': {error}", path.display())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn detects_nul_heavy_binary_data() {
        assert!(is_probably_binary(b"PNG\0binary"));
        assert!(!is_probably_binary(b"normal source code\n"));
    }

    #[test]
    fn decodes_utf16_little_endian() {
        assert_eq!(
            decode_text(&[0xff, 0xfe, b'H', 0, b'i', 0]),
            Some("Hi".to_owned())
        );
    }

    #[test]
    fn collection_honors_root_and_nested_gitignore_rules() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let test_root = std::env::temp_dir().join(format!(
            "code-bundler-gitignore-{}-{unique}",
            std::process::id()
        ));
        let project = test_root.join("project");

        fs::create_dir_all(project.join("ignored-dir")).unwrap();
        fs::create_dir_all(project.join("nested")).unwrap();
        fs::write(
            project.join(".gitignore"),
            b"ignored.txt\nignored-dir/\n*.log\n!important.log\n",
        )
        .unwrap();
        fs::write(project.join("included.txt"), b"included").unwrap();
        fs::write(project.join("ignored.txt"), b"ignored").unwrap();
        fs::write(project.join("ignored-dir").join("file.txt"), b"ignored").unwrap();
        fs::write(project.join("important.log"), b"included by negation").unwrap();
        fs::write(project.join("other.log"), b"ignored").unwrap();
        fs::write(project.join(".hidden"), b"included").unwrap();
        fs::write(
            project.join("nested").join(".gitignore"),
            b"*.tmp\n!keep.tmp\n",
        )
        .unwrap();
        fs::write(project.join("nested").join("drop.tmp"), b"ignored").unwrap();
        fs::write(project.join("nested").join("keep.tmp"), b"included").unwrap();
        fs::write(project.join("nested").join("kept.txt"), b"included").unwrap();

        let (files, warnings) = collect_files(&project, None).unwrap();
        let mut relative = files
            .iter()
            .map(|file| file.relative.clone())
            .collect::<Vec<_>>();
        relative.sort();

        assert!(warnings.is_empty(), "{warnings:?}");
        assert_eq!(
            relative,
            vec![
                ".gitignore",
                ".hidden",
                "important.log",
                "included.txt",
                "nested/.gitignore",
                "nested/keep.tmp",
                "nested/kept.txt",
            ]
        );

        fs::remove_dir_all(test_root).unwrap();
    }

    #[test]
    fn collection_excludes_version_control_metadata() {
        let test_root = test_root("vcs-metadata");
        let project = test_root.join("project");
        fs::create_dir_all(project.join(".git").join("objects")).unwrap();
        fs::write(project.join(".git").join("config"), b"secret remote").unwrap();
        fs::write(project.join("main.rs"), b"fn main() {}\n").unwrap();

        let (files, warnings) = collect_files(&project, None).unwrap();

        assert!(warnings.is_empty(), "{warnings:?}");
        assert_eq!(
            files
                .iter()
                .map(|file| file.relative.as_str())
                .collect::<Vec<_>>(),
            vec!["main.rs"]
        );
        fs::remove_dir_all(test_root).unwrap();
    }

    #[test]
    fn invalid_text_is_marked_as_binary_instead_of_being_corrupted() {
        let test_root = test_root("invalid-text");
        let project = test_root.join("project");
        fs::create_dir_all(&project).unwrap();
        fs::write(project.join("invalid.txt"), [0xff, b't', b'e', b'x', b't']).unwrap();

        let report = extract_folder(&project, false).unwrap();
        let bundle = fs::read_to_string(report.output_path).unwrap();

        assert_eq!(report.binary_files, 1);
        assert!(bundle.contains(BINARY_MARKER));
        assert!(!bundle.contains(char::REPLACEMENT_CHARACTER));
        fs::remove_dir_all(test_root).unwrap();
    }

    fn test_root(name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "code-bundler-{name}-{}-{unique}",
            std::process::id()
        ))
    }
}
