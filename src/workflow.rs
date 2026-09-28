use std::fs::{self, File, OpenOptions};
use std::io::{self, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

use crate::{extractor, output};

const PROMPT_TEMPLATE: &str = include_str!("../sample_prompt.txt");
const REPOSITORY_PLACEHOLDER: &str = "PASTE THE BUNDLED REPOSITORY HERE";
const REQUEST_PLACEHOLDER: &str = "PASTE THE CHANGE REQUEST HERE";

pub(crate) struct GenerationReport {
    pub(crate) bundle_path: PathBuf,
    pub(crate) prompt_path: PathBuf,
    pub(crate) text_files: usize,
    pub(crate) binary_files: usize,
    pub(crate) warnings: Vec<String>,
}

pub(crate) fn generate_project(
    root: &Path,
    request: &str,
    compress: bool,
) -> Result<GenerationReport, String> {
    let output_directory = output::directory()?;
    generate_project_to(root, request, compress, &output_directory)
}

fn generate_project_to(
    root: &Path,
    request: &str,
    compress: bool,
    output_directory: &Path,
) -> Result<GenerationReport, String> {
    if !root.is_dir() {
        return Err(format!("'{}' is not a folder", root.display()));
    }
    fs::create_dir_all(output_directory).map_err(|error| {
        format!(
            "cannot create output folder '{}': {error}",
            output_directory.display()
        )
    })?;

    let project_name = project_name(root);
    let (bundle_path, prompt_path) = unique_output_pair(output_directory, &project_name);
    let bundle_report = extractor::extract_folder_to(root, &bundle_path, compress)?;
    if let Err(error) = write_prompt(&bundle_report.output_path, &prompt_path, request) {
        return match fs::remove_file(&bundle_report.output_path) {
            Ok(()) => Err(error),
            Err(cleanup_error) => Err(format!(
                "{error}; also could not remove the generated bundle '{}': {cleanup_error}",
                bundle_report.output_path.display()
            )),
        };
    }

    Ok(GenerationReport {
        bundle_path: bundle_report.output_path,
        prompt_path,
        text_files: bundle_report.text_files,
        binary_files: bundle_report.binary_files,
        warnings: bundle_report.warnings,
    })
}

fn write_prompt(bundle_path: &Path, prompt_path: &Path, request: &str) -> Result<(), String> {
    let (before_repository, between_values, after_request) = prompt_parts()?;
    let bundle_file = File::open(bundle_path).map_err(|error| {
        format!(
            "cannot read generated bundle '{}': {error}",
            bundle_path.display()
        )
    })?;
    let prompt_file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(prompt_path)
        .map_err(|error| {
            format!(
                "cannot create prompt file '{}': {error}",
                prompt_path.display()
            )
        })?;
    let mut output = BufWriter::new(prompt_file);
    let write_result = (|| {
        output.write_all(before_repository.as_bytes())?;
        io::copy(&mut BufReader::new(bundle_file), &mut output)?;
        output.write_all(between_values.as_bytes())?;
        output.write_all(request.trim().as_bytes())?;
        output.write_all(after_request.as_bytes())?;
        output.flush()
    })();

    if let Err(error) = write_result {
        drop(output);
        let cleanup = fs::remove_file(prompt_path);
        let message = format!("cannot write '{}': {error}", prompt_path.display());
        return match cleanup {
            Ok(()) => Err(message),
            Err(cleanup_error) => Err(format!(
                "{message}; also could not remove the incomplete prompt: {cleanup_error}"
            )),
        };
    }
    Ok(())
}

#[cfg(test)]
fn render_prompt(repository: &str, request: &str) -> Result<String, String> {
    let (before_repository, between_values, after_request) = prompt_parts()?;
    let mut output = String::with_capacity(
        before_repository.len()
            + repository.len()
            + between_values.len()
            + request.len()
            + after_request.len(),
    );
    output.push_str(before_repository);
    output.push_str(repository);
    output.push_str(between_values);
    output.push_str(request.trim());
    output.push_str(after_request);
    Ok(output)
}

fn prompt_parts() -> Result<(&'static str, &'static str, &'static str), String> {
    let (before_repository, after_repository) = PROMPT_TEMPLATE
        .split_once(REPOSITORY_PLACEHOLDER)
        .ok_or_else(|| {
            format!("sample_prompt.txt is missing the '{REPOSITORY_PLACEHOLDER}' placeholder")
        })?;
    let (between_values, after_request) = after_repository
        .split_once(REQUEST_PLACEHOLDER)
        .ok_or_else(|| {
            format!("sample_prompt.txt is missing the '{REQUEST_PLACEHOLDER}' placeholder")
        })?;

    Ok((before_repository, between_values, after_request))
}

fn unique_output_pair(root: &Path, project_name: &str) -> (PathBuf, PathBuf) {
    for number in 1u32.. {
        let suffix = if number == 1 {
            String::new()
        } else {
            format!("_{number}")
        };
        let bundle = root.join(format!("extracted_content_{project_name}{suffix}.txt"));
        let prompt = root.join(format!("generated_prompt_{project_name}{suffix}.txt"));
        if !bundle.exists() && !prompt.exists() {
            return (bundle, prompt);
        }
    }
    unreachable!("the output file counter cannot be exhausted")
}

fn project_name(root: &Path) -> String {
    let raw_name = root
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("project");
    let mut sanitized = String::with_capacity(raw_name.len());
    let mut previous_was_separator = false;

    for character in raw_name.chars() {
        if character.is_alphanumeric() || matches!(character, '-' | '_') {
            sanitized.push(character);
            previous_was_separator = false;
        } else if !previous_was_separator {
            sanitized.push('_');
            previous_was_separator = true;
        }
    }

    let sanitized = sanitized.trim_matches('_');
    if sanitized.is_empty() {
        "project".to_owned()
    } else {
        sanitized.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

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

    #[test]
    fn prompt_includes_the_bundle_and_optional_request() {
        let prompt = render_prompt("==code-bundler:v1==\n", "  Change the title.  ").unwrap();

        assert!(prompt.contains("<repository>\n==code-bundler:v1==\n\n</repository>"));
        assert!(prompt.contains("<request>\nChange the title.\n</request>"));
        assert!(prompt.contains("add_file:<new_relative_path>"));
        assert!(prompt.contains("delete_file:<relative_path>"));
        assert!(prompt.contains("rename_file:<old_relative_path>"));
        assert!(!prompt.contains(REPOSITORY_PLACEHOLDER));
        assert!(!prompt.contains(REQUEST_PLACEHOLDER));
    }

    #[test]
    fn generation_creates_a_named_bundle_and_prompt_without_overwriting() {
        let test_root = test_root("generation");
        let root = test_root.join("sample project");
        let output_directory = test_root.join("Documents").join("code_bundler");
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("main.txt"), "hello\n").unwrap();

        let first = generate_project_to(
            &root,
            "Update the greeting.",
            false,
            &output_directory,
        )
        .unwrap();
        let second =
            generate_project_to(&root, "Update it again.", false, &output_directory).unwrap();

        assert_eq!(
            first.bundle_path.file_name().unwrap(),
            "extracted_content_sample_project.txt"
        );
        assert_eq!(
            first.prompt_path.file_name().unwrap(),
            "generated_prompt_sample_project.txt"
        );
        assert_eq!(
            second.bundle_path.file_name().unwrap(),
            "extracted_content_sample_project_2.txt"
        );
        assert_eq!(
            second.prompt_path.file_name().unwrap(),
            "generated_prompt_sample_project_2.txt"
        );
        assert_eq!(second.text_files, 1, "previous outputs must not be bundled");
        assert_eq!(first.bundle_path.parent(), Some(output_directory.as_path()));
        assert_eq!(first.prompt_path.parent(), Some(output_directory.as_path()));
        assert!(
            fs::read_to_string(first.prompt_path)
                .unwrap()
                .contains("Update the greeting.")
        );

        fs::remove_dir_all(test_root).unwrap();
    }
}
