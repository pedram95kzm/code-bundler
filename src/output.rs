use std::path::PathBuf;

const OUTPUT_DIRECTORY_NAME: &str = "code_bundler";

pub(crate) fn directory() -> Result<PathBuf, String> {
    dirs::document_dir()
        .map(|documents| documents.join(OUTPUT_DIRECTORY_NAME))
        .ok_or_else(|| "cannot locate your Documents folder".to_owned())
}
