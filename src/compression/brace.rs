#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum State {
    Normal,
    SingleQuoted,
    DoubleQuoted,
    BacktickQuoted,
    LineComment,
    BlockComment,
    RawString(usize),
}

/// Collapses whitespace outside literals and comments. It always emits one
/// separator where whitespace existed, so adjacent code tokens cannot merge.
/// Newlines ending line comments are retained because removing those can turn
/// the following code into part of the comment.
pub(super) fn compress(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    let mut state = State::Normal;
    let mut escaped = false;
    let mut pending_whitespace = false;

    while let Some(character) = chars.next() {
        match state {
            State::Normal => {
                if character.is_whitespace() {
                    pending_whitespace = true;
                    continue;
                }

                push_pending_separator(&mut output, &mut pending_whitespace);

                if character == '<' {
                    let mut lookahead = chars.clone();
                    if lookahead.next() == Some('<') && lookahead.next() == Some('<') {
                        // Heredoc/nowdoc bodies are whitespace-sensitive. Keep
                        // this section and everything after it byte-for-byte.
                        output.push(character);
                        output.extend(chars);
                        break;
                    }
                }

                if character == 'R' && raw_string_prefix(&chars).is_some() {
                    // C++ raw strings can use arbitrary delimiters. Keeping the
                    // remainder unchanged is safer than guessing its terminator.
                    output.push(character);
                    output.extend(chars);
                    break;
                }

                if character == 'r'
                    && let Some(hash_count) = raw_string_prefix(&chars)
                {
                    output.push(character);
                    for _ in 0..hash_count {
                        output.push(chars.next().expect("raw prefix hash"));
                    }
                    output.push(chars.next().expect("raw prefix quote"));
                    state = State::RawString(hash_count);
                    continue;
                }

                match character {
                    '\'' => {
                        output.push(character);
                        state = State::SingleQuoted;
                        escaped = false;
                    }
                    '"' => {
                        output.push(character);
                        state = State::DoubleQuoted;
                        escaped = false;
                    }
                    '`' => {
                        output.push(character);
                        state = State::BacktickQuoted;
                        escaped = false;
                    }
                    '/' if chars.peek() == Some(&'/') => {
                        output.push(character);
                        output.push(chars.next().expect("line comment slash"));
                        state = State::LineComment;
                    }
                    '/' if chars.peek() == Some(&'*') => {
                        output.push(character);
                        output.push(chars.next().expect("block comment star"));
                        state = State::BlockComment;
                    }
                    '#' => {
                        output.push(character);
                        state = State::LineComment;
                    }
                    _ => output.push(character),
                }
            }
            State::SingleQuoted | State::DoubleQuoted | State::BacktickQuoted => {
                output.push(character);
                let closing = match state {
                    State::SingleQuoted => '\'',
                    State::DoubleQuoted => '"',
                    State::BacktickQuoted => '`',
                    _ => unreachable!(),
                };
                if escaped {
                    escaped = false;
                } else if character == '\\' {
                    escaped = true;
                } else if character == closing {
                    state = State::Normal;
                }
            }
            State::LineComment => {
                if character == '\r' {
                    if chars.peek() == Some(&'\n') {
                        chars.next();
                    }
                    output.push('\n');
                    state = State::Normal;
                } else if character == '\n' {
                    output.push('\n');
                    state = State::Normal;
                } else {
                    output.push(character);
                }
            }
            State::BlockComment => {
                output.push(character);
                if character == '*' && chars.peek() == Some(&'/') {
                    output.push(chars.next().expect("block comment slash"));
                    state = State::Normal;
                }
            }
            State::RawString(hash_count) => {
                output.push(character);
                if character == '"' && next_chars_are_hashes(&chars, hash_count) {
                    for _ in 0..hash_count {
                        output.push(chars.next().expect("raw suffix hash"));
                    }
                    state = State::Normal;
                }
            }
        }
    }

    output
}

fn push_pending_separator(output: &mut String, pending: &mut bool) {
    if *pending && !output.is_empty() && !output.ends_with('\n') {
        output.push(' ');
    }
    *pending = false;
}

fn raw_string_prefix<I>(chars: &std::iter::Peekable<I>) -> Option<usize>
where
    I: Iterator<Item = char> + Clone,
{
    let mut lookahead = chars.clone();
    let mut hashes = 0usize;
    while lookahead.peek() == Some(&'#') {
        hashes += 1;
        lookahead.next();
    }
    (lookahead.next() == Some('"')).then_some(hashes)
}

fn next_chars_are_hashes<I>(chars: &std::iter::Peekable<I>, count: usize) -> bool
where
    I: Iterator<Item = char> + Clone,
{
    let mut lookahead = chars.clone();
    (0..count).all(|_| lookahead.next() == Some('#'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_strings_and_token_boundaries() {
        assert_eq!(
            compress("  function  test ( ) {\n return  \"a  b\\n\";\n}\n"),
            "function test ( ) { return \"a  b\\n\"; }"
        );
    }

    #[test]
    fn keeps_line_comment_endings() {
        assert_eq!(
            compress("let a = 1; // explanation\n  let b = 2;"),
            "let a = 1; // explanation\nlet b = 2;"
        );
    }

    #[test]
    fn preserves_raw_strings_and_heredocs() {
        assert_eq!(
            compress("let  value = r#\"a   b\n c\"#;\n next();"),
            "let value = r#\"a   b\n c\"#; next();"
        );
        assert_eq!(
            compress("echo   <<<TEXT\n  keep me\nTEXT;\nnext();"),
            "echo <<<TEXT\n  keep me\nTEXT;\nnext();"
        );
    }
}
