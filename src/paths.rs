use std::path::{Component, Path, PathBuf};

const RESERVED_WINDOWS_NAMES: &[&str] = &[
    "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
    "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9", "CONIN$",
    "CONOUT$",
];

pub(crate) fn parse_portable_relative_path(raw_path: &str) -> Result<(PathBuf, String), String> {
    if raw_path.is_empty() {
        return Err("a file path is empty".to_owned());
    }

    let mut path = PathBuf::new();
    let mut key_parts = Vec::new();
    for component in raw_path.split('/') {
        validate_component(component, raw_path)?;
        path.push(component);
        key_parts.push(component.to_lowercase());
    }

    Ok((path, key_parts.join("/")))
}

pub(crate) fn portable_relative_display(root: &Path, path: &Path) -> Result<String, String> {
    let relative = path.strip_prefix(root).map_err(|_| {
        format!(
            "'{}' is outside the selected project folder",
            path.display()
        )
    })?;
    let mut parts = Vec::new();

    for component in relative.components() {
        let Component::Normal(value) = component else {
            return Err(format!(
                "'{}' does not have a safe relative path",
                path.display()
            ));
        };
        let value = value.to_str().ok_or_else(|| {
            format!(
                "'{}' has a file name that is not valid Unicode",
                path.display()
            )
        })?;
        validate_component(value, &relative.display().to_string())?;
        parts.push(value);
    }

    if parts.is_empty() {
        return Err(format!("'{}' is not a file path", path.display()));
    }
    Ok(parts.join("/"))
}

pub(crate) fn portable_path_key(path: &Path) -> Result<String, String> {
    let mut parts = Vec::new();
    for component in path.components() {
        let Component::Normal(value) = component else {
            return Err(format!(
                "'{}' is not a safe relative file path",
                path.display()
            ));
        };
        let value = value
            .to_str()
            .ok_or_else(|| format!("'{}' is not a valid Unicode file path", path.display()))?;
        validate_component(value, &path.display().to_string())?;
        parts.push(value.to_lowercase());
    }

    if parts.is_empty() {
        return Err("a file path is empty".to_owned());
    }
    Ok(parts.join("/"))
}

fn validate_component(component: &str, full_path: &str) -> Result<(), String> {
    let has_forbidden_character = component.chars().any(|character| {
        character.is_control()
            || matches!(character, '<' | '>' | ':' | '"' | '\\' | '|' | '?' | '*')
    });
    let basename = component.split('.').next().unwrap_or(component);
    let reserved = RESERVED_WINDOWS_NAMES
        .iter()
        .any(|name| basename.eq_ignore_ascii_case(name));

    if component.is_empty()
        || matches!(component, "." | "..")
        || component.ends_with([' ', '.'])
        || has_forbidden_character
        || reserved
    {
        return Err(format!("unsafe or non-portable file path: '{full_path}'"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_normal_portable_paths() {
        let (path, key) = parse_portable_relative_path("Src/Main.rs").unwrap();

        assert_eq!(path, Path::new("Src/Main.rs"));
        assert_eq!(key, "src/main.rs");
    }

    #[test]
    fn rejects_traversal_and_non_portable_names() {
        for path in [
            "../outside.txt",
            "/absolute.txt",
            "folder\\file.txt",
            "folder/con.txt",
            "trailing-dot.",
            "line\nbreak.txt",
        ] {
            assert!(
                parse_portable_relative_path(path).is_err(),
                "'{path}' should be rejected"
            );
        }
    }
}
