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
    Long,
    Void,
    Indeterminate,
}

impl DataType {
    pub fn size(&self) -> usize {
        match self {
            DataType::Int => 4,
            DataType::Long => 8,
            _ => panic!("Size not defined for this data type"),
        }
    }

    /// this always returns the super type of the two data types.
    /// if the two types are not compatible , it returns an error.
    pub fn common(&self, other: &DataType, span: Span) -> Result<DataType> {
        match (self, other) {
            (a, b) if a == b => Ok(*a),
            (DataType::Int, DataType::Long) | (DataType::Long, DataType::Int) => Ok(DataType::Long),
            _ => Err(Error::semantic(format!(
                "Cannot use {} and {} together", self, other
            ), span)),
        }
    }
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
    Unspecified,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InitVal {
    Const(Constant),
    Tentative,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Constant {
    Int(i32),
    Long(i64),
}

impl Display for Constant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Constant::Int(value) => write!(f, "{}", value),
            Constant::Long(value) => write!(f, "{}", value),
        }
    }
}

impl Display for DataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataType::Int => write!(f, "int"),
            DataType::Long => write!(f, "long"),
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
            StorageClass::Unspecified => write!(f, ""),
        }
    }
}