use crate::embedder;
use crate::extractor;
use crate::prompt;

pub(crate) fn run() -> Result<(), String> {
    println!("Code Bundler");
    println!("Extract a folder to text, or embed extracted text back into files.\n");
    println!("Tip: press Tab to show matching paths.");

    let choices = prompt::read_user_choices()?;
    let path = std::path::absolute(&choices.path)
        .map_err(|error| format!("cannot resolve '{}': {error}", choices.path.display()))?;

    if path.is_dir() {
        return extract(&path, choices.compress.unwrap_or(false));
    }
    if path.is_file() {
        return embed(&path);
    }

    Err(format!("'{}' is not a file or folder", path.display()))
}

fn extract(root: &std::path::Path, compress: bool) -> Result<(), String> {
    let report = extractor::extract_folder(root, compress)?;

    println!("\nExtract complete: {} text file(s).", report.text_files);
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

fn embed(input_path: &std::path::Path) -> Result<(), String> {
    let report = embedder::embed_file(input_path)?;

    println!(
        "\nEmbed complete: {} file(s) created.",
        report.files_created
    );
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
