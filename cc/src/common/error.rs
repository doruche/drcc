//! Top-level error module.

use std::fmt::Display;

use crate::{lex, ast};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Lex(String),
    Parse(String),
    Semantic(String),
    // General errors
    Errors(Vec<Error>),
    Unimplemented,
    Other(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Lex(err) => write!(f, "Lexer error: {}", err),
            Error::Parse(err) => write!(f, "Parser error: {}", err),
            Error::Semantic(err) => write!(f, "Semantic error: {}", err),
            Error::Errors(errors) => {
                for (i, error) in errors.iter().enumerate() {
                    write!(f, "{}\n", error)?;
                }
                Ok(())
            }
            Error::Unimplemented => write!(f, "Feature not implemented"),
            Error::Other(msg) => write!(f, "{}", msg),
        }
    }
}
