mod span;
mod token;
mod error;
mod string_pool;

use std::{fmt::Display, ops::{Add, Div, Mul, Rem, Sub}};

pub use span::Span;
pub use token::{RawToken, Token, TokenType};
pub use error::{Error, Result};
pub use string_pool::{StringPool, StrDescriptor};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

    pub fn align(&self) -> usize {
        match self {
            DataType::Int => 4,
            DataType::Long => 8,
            _ => panic!("Alignment not defined for this data type"),
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

    pub fn sized_str(&self) -> &str {
        match self {
            DataType::Int => "i32",
            DataType::Long => "i64",
            _ => unreachable!(),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Constant {
    Int(i32),
    Long(i64),
}

impl Constant {
    pub fn value(&self) -> i64 {
        match self {
            Constant::Int(value) => *value as i64,
            Constant::Long(value) => *value,
        }
    }

    pub fn data_type(&self) -> DataType {
        match self {
            Constant::Int(_) => DataType::Int,
            Constant::Long(_) => DataType::Long,
        }
    }

    pub fn is_zero(&self) -> bool {
        match self {
            Constant::Int(value) => *value == 0,
            Constant::Long(value) => *value == 0,
        }
    }

    pub fn neg(&self) -> Self {
        match self {
            Constant::Int(value) => Constant::Int(-value),
            Constant::Long(value) => Constant::Long(-value),
        }
    }

    pub fn complement(&self) -> Self {
        match self {
            Constant::Int(value) => Constant::Int(!value),
            Constant::Long(value) => Constant::Long(!value),
        }
    }

    pub fn not(&self) -> Self {
        match self {
            Constant::Int(value) => Constant::Int(
                if value == &0 { 1 } else { 0 }
            ),
            Constant::Long(value) => Constant::Long(
                if value == &0 { 1 } else { 0 }
            ),
        }
    }
}

impl Add for Constant {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        match (self, other) {
            (Constant::Int(a), Constant::Int(b)) => Constant::Int(a.wrapping_add(b)),
            (Constant::Long(a), Constant::Long(b)) => Constant::Long(a.wrapping_add(b)),
            _ => panic!("Cannot add constants of different types"),
        }
    }
}

impl Sub for Constant {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        match (self, other) {
            (Constant::Int(a), Constant::Int(b)) => Constant::Int(a.wrapping_sub(b)),
            (Constant::Long(a), Constant::Long(b)) => Constant::Long(a.wrapping_sub(b)),
            _ => panic!("Cannot subtract constants of different types"),
        }
    }
}

impl Mul for Constant {
    type Output = Self;

    fn mul(self, other: Self) -> Self::Output {
        match (self, other) {
            (Constant::Int(a), Constant::Int(b)) => Constant::Int(a.wrapping_mul(b)),
            (Constant::Long(a), Constant::Long(b)) => Constant::Long(a.wrapping_mul(b)),
            _ => panic!("Cannot multiply constants of different types"),
        }
    }
}

impl Div for Constant {
    type Output = Self;

    fn div(self, other: Self) -> Self::Output {
        match (self, other) {
            (Constant::Int(a), Constant::Int(b)) => {
                if b == 0 {
                    panic!("Division by zero");
                }
                Constant::Int(a.wrapping_div(b))
            }
            (Constant::Long(a), Constant::Long(b)) => {
                if b == 0 {
                    panic!("Division by zero");
                }
                Constant::Long(a.wrapping_div(b))
            }
            _ => panic!("Cannot divide constants of different types"),
        }
    }
}

impl Rem for Constant {
    type Output = Self;

    fn rem(self, other: Self) -> Self::Output {
        match (self, other) {
            (Constant::Int(a), Constant::Int(b)) => {
                if b == 0 {
                    panic!("Division by zero");
                }
                Constant::Int(a.wrapping_rem(b))
            }
            (Constant::Long(a), Constant::Long(b)) => {
                if b == 0 {
                    panic!("Division by zero");
                }
                Constant::Long(a.wrapping_rem(b))
            }
            _ => panic!("Cannot modulo constants of different types"),
        }
    }
}

impl PartialOrd for Constant {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Constant::Int(a), Constant::Int(b)) => a.partial_cmp(b),
            (Constant::Long(a), Constant::Long(b)) => a.partial_cmp(b),
            _ => None, // Different types cannot be compared
        }
    }
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