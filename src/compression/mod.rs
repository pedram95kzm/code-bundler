mod brace;
mod python;

use std::path::Path;

pub(crate) fn safely_compress(path: &Path, input: &str) -> String {
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .map(str::to_ascii_lowercase);

    match extension.as_deref() {
        // Python indentation and logical newlines are syntax. Its dedicated
        // compactor preserves both while reducing safe whitespace.
        Some("py" | "pyi" | "pyw") => python::compress(input),

        // These languages delimit blocks/statements independently of ordinary
        // whitespace. Their compactor still preserves lexical boundaries.
        Some(
            "php" | "php3" | "php4" | "php5" | "php7" | "php8" | "phtml" | "inc" | "c" | "h" | "cc"
            | "cpp" | "cxx" | "hxx" | "hpp" | "cs" | "java" | "kt" | "kts" | "rs" | "css" | "scss"
            | "less" | "json",
        ) => brace::compress(input),

        // Unknown or whitespace-sensitive formats remain byte-for-byte text.
        _ => input.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_formats_are_left_unchanged() {
        let source = "key:  value\n  nested: item\n";
        assert_eq!(safely_compress(Path::new("settings.yml"), source), source);
    }
}
