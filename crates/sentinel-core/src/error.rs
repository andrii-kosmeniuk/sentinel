use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    EmptyInput,
    MissingExecutable,
    UnterminatedQuote { quote: char },
    UnexpectedOperator { operator: String },
    MissingCommandAfterSudo,
    UnsupportedSyntax { syntax: String },
}

impl fmt::Display for ParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyInput => write!(formatter, "command input is empty"),
            Self::MissingExecutable => write!(formatter, "command executable is empty"),
            Self::UnterminatedQuote { quote } => {
                write!(formatter, "unterminated {quote} quote")
            }
            Self::UnexpectedOperator { operator } => {
                write!(formatter, "unexpected shell operator: {operator}")
            }
            Self::MissingCommandAfterSudo => {
                write!(formatter, "sudo must be followed by a command")
            }
            Self::UnsupportedSyntax { syntax } => {
                write!(formatter, "unsupported shell syntax: {syntax}")
            }
        }
    }
}

impl Error for ParseError {}
