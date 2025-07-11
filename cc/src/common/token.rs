//! Token, and tokens' types used in relevant stages of the compiler.

use crate::common::span::Span;

/// Token without any additional information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RawToken {
    // no-lexeme tokens
    LParen, RParen,
    LBrace, RBrace,
    Semicolon,
    Return, Int, Void,


    // [0-9]+
    Integer(i64),
    // [a-zA-Z_][a-zA-Z0-9_]*
    Identifier(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub inner: RawToken,
    pub span: Span,
}

impl Token {
    pub fn new(inner: RawToken, span: Span) -> Self {
        Token { inner, span }
    }
}