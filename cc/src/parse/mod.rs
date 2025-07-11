//! Parsing module.

use std::fmt::Display;

#[derive(Debug)]
pub enum Error {
    Other(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Other(msg) => write!(f, "{}", msg),
        }
    }
}