mod span;
mod token;
mod error;
mod string_pool;

use std::fmt::Display;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageClass {
    Static,
    Extern,
}

impl Display for DataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataType::Int => write!(f, "int"),
            DataType::Void => write!(f, "void"),
            DataType::Indeterminate => write!(f, "indeterminate"),
        }
    }
}

impl Display for Linkage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Linkage::Internal => write!(f, "internal"),
            Linkage::External => write!(f, "external"),
        }
    }
}

impl Display for StorageClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StorageClass::Static => write!(f, "static"),
            StorageClass::Extern => write!(f, "extern"),
        }
    }
}