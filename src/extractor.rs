use std::fs::{self, File, OpenOptions};
use std::io::{self, BufWriter, Read, Write};
use std::path::{Path, PathBuf};

use ignore::WalkBuilder;

use crate::compression;

const OUTPUT_NAME: &str = "extracted_contents.txt";
pub(crate) const FORMAT_MARKER: &str = "==code-bundler:v2==";
pub(crate) const PREVIOUS_FORMAT_MARKER: &str = "==code-extractor:v2==";
pub(crate) const LENGTH_PREFIX: &str = "==utf8-bytes:";
pub(crate) const METADATA_SUFFIX: &str = "==";
pub(crate) const BINARY_MARKER: &str = "[Skipped: binary file]";

pub(crate) struct ExtractionReport {
    pub(crate) output_path: PathBuf,
    pub(crate) text_files: usize,
    pub(crate) binary_files: usize,
    pub(crate) warnings: Vec<String>,
}

pub(crate) fn extract_folder(root: &Path, compress: bool) -> Result<ExtractionReport, String> {
    let (mut files, mut warnings) = collect_files(root)?;
    files.sort_by(|left, right| {
        relative_display(root, left)
            .to_lowercase()
            .cmp(&relative_display(root, right).to_lowercase())
    });

    let (output_path, output_file) = create_unique_output(root)?;
    let mut output = BufWriter::new(output_file);
    let mut text_files = 0usize;
    let mut binary_files = 0usize;

    writeln!(output, "{FORMAT_MARKER}").map_err(|error| output_error(&output_path, error))?;

    for path in files {
        let relative = relative_display(root, &path);
        let mut contents = Vec::new();
        match File::open(&path).and_then(|mut file| file.read_to_end(&mut contents)) {
            Ok(_) => {
                writeln!(output, "=={relative} content==")
                    .map_err(|error| output_error(&output_path, error))?;

                if is_probably_binary(&contents) {
                    writeln!(output, "{BINARY_MARKER}\n")
                        .map_err(|error| output_error(&output_path, error))?;
                    binary_files += 1;
                } else {
                    let decoded = decode_text(&contents);
                    let text = if compress {
                        compression::safely_compress(&path, &decoded)
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
        warnings,
    })
}

fn collect_files(root: &Path) -> Result<(Vec<PathBuf>, Vec<String>), String> {
    fs::read_dir(root)
        .map_err(|error| format!("cannot read folder '{}': {error}", root.display()))?;

    let mut files = Vec::new();
    let mut warnings = Vec::new();

    let mut builder = WalkBuilder::new(root);
    builder
        .hidden(false)
        .parents(false)
        .ignore(false)
        .git_global(false)
        .git_exclude(false)
        .git_ignore(true)
        .require_git(false)
        .follow_links(false);

    for result in builder.build() {
        match result {
            Ok(entry) => {
                if let Some(error) = entry.error() {
                    warnings.push(format!("{}: {error}", entry.path().display()));
                }
                if entry.file_type().is_some_and(|kind| kind.is_file()) {
                    files.push(entry.into_path());
                }
            }
            Err(error) => warnings.push(error.to_string()),
        }
    }

    Ok((files, warnings))
}

fn create_unique_output(root: &Path) -> Result<(PathBuf, File), String> {
    for number in 1u32.. {
        let name = if number == 1 {
            OUTPUT_NAME.to_string()
        } else {
            format!("extracted_contents_{number}.txt")
        };
        let path = root.join(name);
        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(file) => return Ok((path, file)),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
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

fn relative_display(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .components()
        .map(|part| part.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

fn is_probably_binary(bytes: &[u8]) -> bool {
    bytes.iter().take(8_192).any(|byte| *byte == 0)
        && !bytes.starts_with(&[0xff, 0xfe])
        && !bytes.starts_with(&[0xfe, 0xff])
}

pub(crate) fn decode_text(bytes: &[u8]) -> String {
    if bytes.starts_with(&[0xff, 0xfe]) {
        let units = bytes[2..]
            .chunks_exact(2)
            .map(|pair| u16::from_le_bytes([pair[0], pair[1]]));
        return char::decode_utf16(units)
            .map(|character| character.unwrap_or(char::REPLACEMENT_CHARACTER))
            .collect();
    }
    if bytes.starts_with(&[0xfe, 0xff]) {
        let units = bytes[2..]
            .chunks_exact(2)
            .map(|pair| u16::from_be_bytes([pair[0], pair[1]]));
        return char::decode_utf16(units)
            .map(|character| character.unwrap_or(char::REPLACEMENT_CHARACTER))
            .collect();
    }

    String::from_utf8_lossy(bytes.strip_prefix(&[0xef, 0xbb, 0xbf]).unwrap_or(bytes)).into_owned()
}

fn output_error(path: &Path, error: io::Error) -> String {
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
        assert_eq!(decode_text(&[0xff, 0xfe, b'H', 0, b'i', 0]), "Hi");
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

        let (files, warnings) = collect_files(&project).unwrap();
        let mut relative = files
            .iter()
            .map(|path| relative_display(&project, path))
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
}
