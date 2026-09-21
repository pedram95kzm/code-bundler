use std::io::{self, Write};
use std::path::PathBuf;

use rustyline::completion::{Completer, FilenameCompleter, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{CompletionType, Config, Context, Editor, Helper};

struct PathHelper {
    completer: PromptCompleter,
}

enum PromptCompleter {
    Choices(&'static [&'static str]),
    Paths(FilenameCompleter),
}

impl Completer for PathHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        context: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        match &self.completer {
            PromptCompleter::Choices(choices) => Ok(complete_choices(line, pos, choices)),
            PromptCompleter::Paths(completer) => completer.complete(line, pos, context),
        }
    }
}

impl Hinter for PathHelper {
    type Hint = String;
}

impl Highlighter for PathHelper {}
impl Validator for PathHelper {}
impl Helper for PathHelper {}

pub(crate) enum UserChoices {
    Embed {
        folder_path: PathBuf,
        compress: bool,
    },
    Extract {
        text_path: PathBuf,
        modification_path: Option<PathBuf>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Operation {
    Embed,
    Extract,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ExtractionMode {
    Raw,
    WithModification,
}

const OPERATION_CHOICES: &[&str] = &["embed", "extract"];
const COMPRESSION_CHOICES: &[&str] = &["yes", "no"];
const EXTRACTION_MODE_CHOICES: &[&str] = &["raw", "with_modification"];

pub(crate) fn read_user_choices() -> Result<UserChoices, String> {
    let config = Config::builder()
        .completion_type(CompletionType::List)
        .completion_show_all_if_ambiguous(true)
        .build();
    let mut editor = Editor::<PathHelper, rustyline::history::DefaultHistory>::with_config(config)
        .map_err(|error| format!("could not initialize the interactive prompt: {error}"))?;

    enable_choice_completion(&mut editor, OPERATION_CHOICES);
    match ask_operation(&mut editor)? {
        Operation::Embed => {
            enable_choice_completion(&mut editor, COMPRESSION_CHOICES);
            let compress = ask_compression(&mut editor)?;
            enable_path_completion(&mut editor);
            let folder_path = ask_path(&mut editor, "Folder to embed into a text file: ")?;
            Ok(UserChoices::Embed {
                folder_path,
                compress,
            })
        }
        Operation::Extract => {
            enable_choice_completion(&mut editor, EXTRACTION_MODE_CHOICES);
            let mode = ask_extraction_mode(&mut editor)?;
            enable_path_completion(&mut editor);
            let modification_path = if mode == ExtractionMode::WithModification {
                Some(ask_path(&mut editor, "Modification file: ")?)
            } else {
                None
            };
            let text_path = ask_path(&mut editor, "Bundled .txt file to extract: ")?;
            Ok(UserChoices::Extract {
                text_path,
                modification_path,
            })
        }
    }
}

fn enable_path_completion(editor: &mut Editor<PathHelper, rustyline::history::DefaultHistory>) {
    editor.set_helper(Some(PathHelper {
        completer: PromptCompleter::Paths(FilenameCompleter::new()),
    }));
}

fn enable_choice_completion(
    editor: &mut Editor<PathHelper, rustyline::history::DefaultHistory>,
    choices: &'static [&'static str],
) {
    editor.set_helper(Some(PathHelper {
        completer: PromptCompleter::Choices(choices),
    }));
}

fn complete_choices(line: &str, pos: usize, choices: &[&str]) -> (usize, Vec<Pair>) {
    let input = &line[..pos];
    let start = input
        .find(|character: char| !character.is_whitespace())
        .unwrap_or(pos);
    let prefix = input[start..].to_ascii_lowercase();
    let candidates = choices
        .iter()
        .filter(|choice| choice.starts_with(&prefix))
        .map(|choice| Pair {
            display: (*choice).to_string(),
            replacement: (*choice).to_string(),
        })
        .collect();
    (start, candidates)
}

fn ask_operation(
    editor: &mut Editor<PathHelper, rustyline::history::DefaultHistory>,
) -> Result<Operation, String> {
    loop {
        let answer = editor
            .readline("Choose an operation - embed or extract: ")
            .map_err(readline_error)?;

        if let Some(operation) = parse_operation(&answer) {
            return Ok(operation);
        }
        println!("Please enter embed or extract.");
    }
}

fn ask_extraction_mode(
    editor: &mut Editor<PathHelper, rustyline::history::DefaultHistory>,
) -> Result<ExtractionMode, String> {
    loop {
        let answer = editor
            .readline("Extract raw or with_modification? [raw/with_modification]: ")
            .map_err(readline_error)?;

        if let Some(mode) = parse_extraction_mode(&answer) {
            return Ok(mode);
        }
        println!("Please enter raw or with_modification.");
    }
}

fn ask_compression(
    editor: &mut Editor<PathHelper, rustyline::history::DefaultHistory>,
) -> Result<bool, String> {
    loop {
        let answer = editor
            .readline("Compress code in the embedded text file? [y/N]: ")
            .map_err(readline_error)?;

        if let Some(choice) = parse_compression_answer(&answer) {
            return Ok(choice);
        }
        println!("Please answer y/yes or n/no.");
    }
}

fn ask_path(
    editor: &mut Editor<PathHelper, rustyline::history::DefaultHistory>,
    prompt: &str,
) -> Result<PathBuf, String> {
    let input = editor.readline(prompt).map_err(readline_error)?;
    let path = clean_input(&input);
    if path.as_os_str().is_empty() {
        return Err("no path was entered".to_string());
    }
    Ok(path)
}

fn parse_operation(answer: &str) -> Option<Operation> {
    match answer.trim().to_ascii_lowercase().as_str() {
        "embed" => Some(Operation::Embed),
        "extract" => Some(Operation::Extract),
        _ => None,
    }
}

fn parse_extraction_mode(answer: &str) -> Option<ExtractionMode> {
    match answer.trim().to_ascii_lowercase().as_str() {
        "raw" | "r" => Some(ExtractionMode::Raw),
        "with_modification" | "with modification" | "with modifications" | "modification"
        | "modified" | "modify" | "m" => Some(ExtractionMode::WithModification),
        _ => None,
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
    print!("Press Enter to close...");
    let _ = io::stdout().flush();
    let _ = io::stdin().read_line(&mut String::new());
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
    fn operation_answers_are_parsed() {
        assert_eq!(parse_operation(" Embed "), Some(Operation::Embed));
        assert_eq!(parse_operation("extract"), Some(Operation::Extract));
        assert_eq!(parse_operation("e"), None);
    }

    #[test]
    fn fixed_answers_are_completed_from_the_typed_prefix() {
        let (start, candidates) =
            complete_choices("  with_", "  with_".len(), EXTRACTION_MODE_CHOICES);

        assert_eq!(start, 2);
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].replacement, "with_modification");

        let (_, candidates) = complete_choices("", 0, OPERATION_CHOICES);
        assert_eq!(candidates.len(), 2);
        assert_eq!(candidates[0].replacement, "embed");
        assert_eq!(candidates[1].replacement, "extract");
    }

    #[test]
    fn extraction_modes_are_parsed() {
        assert_eq!(parse_extraction_mode("raw"), Some(ExtractionMode::Raw));
        assert_eq!(parse_extraction_mode("R"), Some(ExtractionMode::Raw));
        assert_eq!(
            parse_extraction_mode("modified"),
            Some(ExtractionMode::WithModification)
        );
        assert_eq!(
            parse_extraction_mode("with_modification"),
            Some(ExtractionMode::WithModification)
        );
        assert_eq!(parse_extraction_mode("other"), None);
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
