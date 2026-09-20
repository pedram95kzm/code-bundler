#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum State {
    Normal,
    SingleQuoted,
    DoubleQuoted,
    TripleSingleQuoted,
    TripleDoubleQuoted,
    Comment,
}

/// Preserves Python indentation and logical newlines while removing blank
/// physical lines and redundant intra-line whitespace outside literals.
pub(super) fn compress(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    let mut state = State::Normal;
    let mut escaped = false;
    let mut at_line_start = true;
    let mut line_has_content = false;
    let mut pending_leading = String::new();
    let mut pending_separator = false;

    while let Some(character) = chars.next() {
        match state {
            State::Normal => {
                if matches!(character, '\r' | '\n') {
                    let newline = take_newline(character, &mut chars);
                    if line_has_content {
                        output.push_str(newline);
                    }
                    pending_leading.clear();
                    pending_separator = false;
                    at_line_start = true;
                    line_has_content = false;
                    continue;
                }

                if character.is_whitespace() {
                    if at_line_start {
                        pending_leading.push(character);
                    } else {
                        pending_separator = true;
                    }
                    continue;
                }

                if at_line_start {
                    output.push_str(&pending_leading);
                    pending_leading.clear();
                    at_line_start = false;
                } else if pending_separator {
                    output.push(' ');
                }
                pending_separator = false;
                line_has_content = true;

                if matches!(character, '\'' | '"') && next_two_are(&chars, character) {
                    output.push(character);
                    output.push(chars.next().expect("second triple quote"));
                    output.push(chars.next().expect("third triple quote"));
                    state = if character == '\'' {
                        State::TripleSingleQuoted
                    } else {
                        State::TripleDoubleQuoted
                    };
                    escaped = false;
                } else {
                    output.push(character);
                    state = match character {
                        '\'' => State::SingleQuoted,
                        '"' => State::DoubleQuoted,
                        '#' => State::Comment,
                        _ => State::Normal,
                    };
                    escaped = false;
                }
            }
            State::Comment => {
                if matches!(character, '\r' | '\n') {
                    output.push_str(take_newline(character, &mut chars));
                    state = State::Normal;
                    at_line_start = true;
                    line_has_content = false;
                } else {
                    output.push(character);
                }
            }
            State::SingleQuoted | State::DoubleQuoted => {
                output.push(character);
                let closing = if state == State::SingleQuoted {
                    '\''
                } else {
                    '"'
                };
                if escaped {
                    escaped = false;
                } else if character == '\\' {
                    escaped = true;
                } else if character == closing {
                    state = State::Normal;
                }
            }
            State::TripleSingleQuoted | State::TripleDoubleQuoted => {
                output.push(character);
                let closing = if state == State::TripleSingleQuoted {
                    '\''
                } else {
                    '"'
                };
                if escaped {
                    escaped = false;
                } else if character == '\\' {
                    escaped = true;
                } else if character == closing && next_two_are(&chars, closing) {
                    output.push(chars.next().expect("second closing triple quote"));
                    output.push(chars.next().expect("third closing triple quote"));
                    state = State::Normal;
                }
            }
        }
    }

    output
}

fn next_two_are<I>(chars: &std::iter::Peekable<I>, expected: char) -> bool
where
    I: Iterator<Item = char> + Clone,
{
    let mut lookahead = chars.clone();
    lookahead.next() == Some(expected) && lookahead.next() == Some(expected)
}

fn take_newline<I>(first: char, chars: &mut std::iter::Peekable<I>) -> &'static str
where
    I: Iterator<Item = char>,
{
    if first == '\r' && chars.peek() == Some(&'\n') {
        chars.next();
        "\r\n"
    } else if first == '\r' {
        "\r"
    } else {
        "\n"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_indentation_and_multiline_strings() {
        let source = concat!(
            "\n",
            "def  greet ( name ):\n",
            "    message  =  f\"hello  {name}\"\n",
            "\n",
            "    text = \"\"\"first\n",
            "  significant spaces\n",
            "\n",
            "third\"\"\"\n",
            "    return  message  # keep  comment\n",
        );
        let expected = concat!(
            "def greet ( name ):\n",
            "    message = f\"hello  {name}\"\n",
            "    text = \"\"\"first\n",
            "  significant spaces\n",
            "\n",
            "third\"\"\"\n",
            "    return message # keep  comment\n",
        );
        assert_eq!(compress(source), expected);
    }
}
