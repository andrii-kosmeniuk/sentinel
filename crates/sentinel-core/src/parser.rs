use std::path::Path;

use crate::error::ParseError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedCommandLine {
    pub commands: Vec<ParsedCommand>,
    pub operators: Vec<ShellOperator>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedCommand {
    pub executable: String,
    pub args: Vec<String>,
    pub uses_sudo: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellOperator {
    Pipe,
    And,
    Sequence,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Quote {
    Single,
    Double,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    Word(String),
    Operator(ShellOperator),
}

pub fn parse_command(input: &str) -> Result<ParsedCommandLine, ParseError> {
    let tokens = tokenize(input)?;
    build_command_line(tokens)
}

fn tokenize(input: &str) -> Result<Vec<Token>, ParseError> {
    if input.trim().is_empty() {
        return Err(ParseError::EmptyInput);
    }

    let mut tokens = Vec::new();
    let mut word = String::new();
    let mut word_started = false;
    let mut quote = None;
    let mut chars = input.chars().peekable();

    while let Some(character) = chars.next() {
        if let Some(active_quote) = quote {
            match (active_quote, character) {
                (Quote::Single, '\'') | (Quote::Double, '"') => quote = None,
                (_, '\n' | '\r') => return unsupported("newline"),
                (_, '$') => return unsupported("$"),
                (_, '`') => return unsupported("`"),
                (_, '\\') => return unsupported("\\"),
                _ => word.push(character),
            }
            continue;
        }

        match character {
            ' ' | '\t' => push_word(&mut tokens, &mut word, &mut word_started),
            '\n' | '\r' => return unsupported("newline"),
            '\'' => {
                word_started = true;
                quote = Some(Quote::Single);
            }
            '"' => {
                word_started = true;
                quote = Some(Quote::Double);
            }
            '|' => {
                if chars.next_if_eq(&'|').is_some() {
                    return unsupported("||");
                }
                push_operator(
                    &mut tokens,
                    &mut word,
                    &mut word_started,
                    ShellOperator::Pipe,
                )?;
            }
            '&' => {
                if chars.next_if_eq(&'&').is_none() {
                    return unsupported("&");
                }
                push_operator(
                    &mut tokens,
                    &mut word,
                    &mut word_started,
                    ShellOperator::And,
                )?;
            }
            ';' => push_operator(
                &mut tokens,
                &mut word,
                &mut word_started,
                ShellOperator::Sequence,
            )?,
            '<' => return unsupported("<"),
            '>' => {
                return unsupported(if chars.next_if_eq(&'>').is_some() {
                    ">>"
                } else {
                    ">"
                });
            }
            '$' => return unsupported("$"),
            '`' => return unsupported("`"),
            '\\' => return unsupported("\\"),
            '(' => return unsupported("("),
            ')' => return unsupported(")"),
            '#' if !word_started => return unsupported("#"),
            _ => {
                word_started = true;
                word.push(character);
            }
        }
    }

    if let Some(active_quote) = quote {
        return Err(ParseError::UnterminatedQuote {
            quote: match active_quote {
                Quote::Single => '\'',
                Quote::Double => '"',
            },
        });
    }

    push_word(&mut tokens, &mut word, &mut word_started);

    if let Some(Token::Operator(operator)) = tokens.last() {
        return Err(ParseError::UnexpectedOperator {
            operator: operator_text(*operator).to_owned(),
        });
    }

    Ok(tokens)
}

fn push_word(tokens: &mut Vec<Token>, word: &mut String, word_started: &mut bool) {
    if *word_started {
        tokens.push(Token::Word(std::mem::take(word)));
        *word_started = false;
    }
}

fn push_operator(
    tokens: &mut Vec<Token>,
    word: &mut String,
    word_started: &mut bool,
    operator: ShellOperator,
) -> Result<(), ParseError> {
    push_word(tokens, word, word_started);

    if tokens.is_empty() || matches!(tokens.last(), Some(Token::Operator(_))) {
        return Err(ParseError::UnexpectedOperator {
            operator: operator_text(operator).to_owned(),
        });
    }

    tokens.push(Token::Operator(operator));
    Ok(())
}

fn build_command_line(tokens: Vec<Token>) -> Result<ParsedCommandLine, ParseError> {
    let mut commands = Vec::new();
    let mut operators = Vec::new();
    let mut words = Vec::new();

    for token in tokens {
        match token {
            Token::Word(word) => words.push(word),
            Token::Operator(operator) => {
                commands.push(build_parsed_command(std::mem::take(&mut words))?);
                operators.push(operator);
            }
        }
    }

    commands.push(build_parsed_command(words)?);

    Ok(ParsedCommandLine {
        commands,
        operators,
    })
}

fn build_parsed_command(words: Vec<String>) -> Result<ParsedCommand, ParseError> {
    if words.first().is_none_or(|word| word.is_empty()) {
        return Err(ParseError::MissingExecutable);
    }

    let uses_sudo = normalize_executable(&words[0]) == "sudo";
    let executable_index = usize::from(uses_sudo);

    if uses_sudo && words.len() == 1 {
        return Err(ParseError::MissingCommandAfterSudo);
    }

    if uses_sudo && words[executable_index].starts_with('-') {
        return unsupported("sudo options");
    }

    let executable = normalize_executable(&words[executable_index]);

    if executable.is_empty() {
        return Err(ParseError::MissingExecutable);
    }

    if uses_sudo && executable == "sudo" {
        return unsupported("nested sudo");
    }

    let args = words[(executable_index + 1)..].to_vec();

    Ok(ParsedCommand {
        executable,
        args,
        uses_sudo,
    })
}

fn normalize_executable(executable: &str) -> String {
    Path::new(executable)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(executable)
        .to_owned()
}

fn unsupported<T>(syntax: &str) -> Result<T, ParseError> {
    Err(ParseError::UnsupportedSyntax {
        syntax: syntax.to_owned(),
    })
}

fn operator_text(operator: ShellOperator) -> &'static str {
    match operator {
        ShellOperator::Pipe => "|",
        ShellOperator::And => "&&",
        ShellOperator::Sequence => ";",
    }
}
