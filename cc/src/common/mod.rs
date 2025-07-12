mod span;
mod token;
mod error;

pub use span::Span;
pub use token::{RawToken, Token, TokenType};
pub use error::{Error, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataType {
    Int,
    Void,
}