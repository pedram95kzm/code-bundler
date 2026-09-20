use std::env;
use std::io::{self, Write};
use std::path::PathBuf;

use rustyline::completion::{Completer, FilenameCompleter, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{CompletionType, Config, Context, Editor, Helper};

struct PathHelper {
    completer: FilenameCompleter,
}

impl Completer for PathHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        context: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        self.completer.complete(line, pos, context)
    }
}

impl Hinter for PathHelper {
    type Hint = String;
}

impl Highlighter for PathHelper {}
impl Validator for PathHelper {}
impl Helper for PathHelper {}

pub(crate) struct UserChoices {
    pub(crate) path: PathBuf,
    pub(crate) compress: Option<bool>,
}

pub(crate) fn read_user_choices() -> Result<UserChoices, String> {
    let config = Config::builder()
        .completion_type(CompletionType::List)
        .completion_show_all_if_ambiguous(true)
        .build();
    let mut editor = Editor::<PathHelper, rustyline::history::DefaultHistory>::with_config(config)
        .map_err(|error| format!("could not initialize the interactive prompt: {error}"))?;
    editor.set_helper(Some(PathHelper {
        completer: FilenameCompleter::new(),
    }));

    let input = editor
        .readline("Folder to extract, or extracted .txt to embed: ")
        .map_err(readline_error)?;
    let path = clean_input(&input);
    if path.as_os_str().is_empty() {
        return Err("no folder path was entered".to_string());
    }

    let compress = if path.is_dir() {
        editor.set_helper(None);
        Some(ask_compression(&mut editor)?)
    } else {
        None
    };
    Ok(UserChoices { path, compress })
}

fn ask_compression(
    editor: &mut Editor<PathHelper, rustyline::history::DefaultHistory>,
) -> Result<bool, String> {
    loop {
        let answer = editor
            .readline("Compress code in the extracted file? [y/N]: ")
            .map_err(readline_error)?;

        if let Some(choice) = parse_compression_answer(&answer) {
            return Ok(choice);
        }
        println!("Please answer y/yes or n/no.");
    }
}

fn parse_compression_answer(answer: &str) -> Option<bool> {
    match answer.trim().to_ascii_lowercase().as_str() {
        "y" | "yes" => Some(true),
        "" | "n" | "no" => Some(false),
        _ => None,
    }
}

fn clean_input(input: &str) -> PathBuf {
    let trimmed = input.trim();

    // Windows path completion adds a leading quote for paths containing spaces.
    // Accept both a complete quote pair and that completion-friendly form.
    let unquoted = strip_optional_quotes(trimmed, '"');
    PathBuf::from(strip_optional_quotes(unquoted, '\''))
}

fn strip_optional_quotes(input: &str, quote: char) -> &str {
    input
        .strip_prefix(quote)
        .unwrap_or(input)
        .strip_suffix(quote)
        .unwrap_or_else(|| input.strip_prefix(quote).unwrap_or(input))
}

fn readline_error(error: ReadlineError) -> String {
    match error {
        ReadlineError::Interrupted => "input was cancelled".to_string(),
        ReadlineError::Eof => "no input was provided".to_string(),
        other => format!("could not read input: {other}"),
    }
}

pub(crate) fn wait_before_exit() {
    // Keep errors visible when the executable is opened by double-clicking.
    if env::args_os().len() == 1 && env::var_os("PROMPT").is_none() {
        eprint!("Press Enter to close...");
        let _ = io::stderr().flush();
        let _ = io::stdin().read_line(&mut String::new());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn input_may_be_quoted() {
        assert_eq!(
            clean_input("  \"C:\\some folder\"\r\n"),
            PathBuf::from("C:\\some folder")
        );
        assert_eq!(
            clean_input("\"C:\\some folder"),
            PathBuf::from("C:\\some folder")
        );
    }

    #[test]
    fn compression_answers_are_parsed() {
        assert_eq!(parse_compression_answer("yes"), Some(true));
        assert_eq!(parse_compression_answer(" Y "), Some(true));
        assert_eq!(parse_compression_answer("no"), Some(false));
        assert_eq!(parse_compression_answer(""), Some(false));
        assert_eq!(parse_compression_answer("maybe"), None);
    }
}
