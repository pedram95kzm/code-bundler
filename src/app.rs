use crate::embedder;
use crate::extractor;
use crate::prompt;

pub(crate) fn run() -> Result<(), String> {
    println!("Code Bundler");
    println!("Embed a folder into text, or extract bundled text back into files.\n");
    println!("Tip: press Tab to autocomplete answers and paths.");

    let choices = prompt::read_user_choices()?;
    match choices {
        prompt::UserChoices::Embed {
            folder_path,
            compress,
        } => {
            let folder_path = absolute_path(&folder_path)?;
            if !folder_path.is_dir() {
                return Err(format!("'{}' is not a folder", folder_path.display()));
            }
            embed(&folder_path, compress)
        }
        prompt::UserChoices::Extract {
            text_path,
            modification_path,
        } => {
            let text_path = absolute_path(&text_path)?;
            if !text_path.is_file() {
                return Err(format!("'{}' is not a file", text_path.display()));
            }
            let modification_path = modification_path
                .as_deref()
                .map(absolute_path)
                .transpose()?;
            if let Some(path) = &modification_path {
                if !path.is_file() {
                    return Err(format!("'{}' is not a modification file", path.display()));
                }
            }
            extract(&text_path, modification_path.as_deref())
        }
    }
}

fn absolute_path(path: &std::path::Path) -> Result<std::path::PathBuf, String> {
    std::path::absolute(path)
        .map_err(|error| format!("cannot resolve '{}': {error}", path.display()))
}

fn embed(root: &std::path::Path, compress: bool) -> Result<(), String> {
    let report = extractor::extract_folder(root, compress)?;

    println!(
        "\nEmbed complete: {} text file(s) bundled.",
        report.text_files
    );
    if compress {
        println!("Safe whitespace compression was applied where language rules allow it.");
    } else {
        println!("Compression was not applied.");
    }
    println!("Your original files were not changed.");

    if report.binary_files > 0 {
        println!("Skipped {} binary file(s).", report.binary_files);
    }
    if !report.warnings.is_empty() {
        println!(
            "Warning: {} item(s) could not be read.",
            report.warnings.len()
        );
        for warning in &report.warnings {
            eprintln!("  {warning}");
        }
    }

    println!("Output: {}", report.output_path.display());
    Ok(())
}

fn extract(
    input_path: &std::path::Path,
    modification_path: Option<&std::path::Path>,
) -> Result<(), String> {
    let report = embedder::embed_file(input_path, modification_path)?;

    println!(
        "\nExtract complete: {} file(s) created.",
        report.files_created
    );
    if report.modifications_applied > 0 {
        println!(
            "Applied {} modification(s) across {} file(s).",
            report.modifications_applied, report.files_modified
        );
    } else {
        println!("Raw extraction was used; no modifications were applied.");
    }
    if report.binary_files_skipped > 0 {
        println!(
            "Skipped {} binary file marker(s); extracted text does not contain their data.",
            report.binary_files_skipped
        );
    }
    if report.legacy_format {
        println!("Note: this is a legacy extract, so final-newline information may not be exact.");
    }
    println!("Output folder: {}", report.output_root.display());
    Ok(())
}
