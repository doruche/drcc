mod span;
mod token;
mod error;
mod string_pool;

pub use span::Span;
pub use token::{RawToken, Token, TokenType};
pub use error::{Error, Result};
pub use string_pool::{StringPool, StrDescriptor};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataType {
    Int,
    Void,
    Indeterminate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FuncType {
    pub return_type: DataType,
    pub param_types: Vec<DataType>,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Linkage {
    Internal,
    External,
}