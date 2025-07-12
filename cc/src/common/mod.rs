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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Pos,
    Negate,
    Complement,
}

#[derive(Debug, Clone, Copy)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem
}

impl BinaryOp {
    pub fn precedence(&self) -> u8 {
        match self {
            BinaryOp::Add | BinaryOp::Sub => 3,
            BinaryOp::Mul | BinaryOp::Div | BinaryOp::Rem => 7,
        }
    }

    pub fn prior_than(&self, other: &BinaryOp) -> bool {
        self.precedence() > other.precedence()
    }
}