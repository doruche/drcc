//! Token, and tokens' types used in relevant stages of the compiler.

use crate::{common::span::Span, span};

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

    Nothing,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenType {
    LParen,
    RParen,
    LBrace,
    RBrace,
    Semicolon,
    Return,
    Int,
    Void,
    Integer,
    Identifier,

    Nothing,
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

    pub fn get_type(&self) -> TokenType {
        match &self.inner {
            RawToken::LParen => TokenType::LParen,
            RawToken::RParen => TokenType::RParen,
            RawToken::LBrace => TokenType::LBrace,
            RawToken::RBrace => TokenType::RBrace,
            RawToken::Semicolon => TokenType::Semicolon,
            RawToken::Return => TokenType::Return,
            RawToken::Int => TokenType::Int,
            RawToken::Void => TokenType::Void,
            RawToken::Integer(_) => TokenType::Integer,
            RawToken::Identifier(_) => TokenType::Identifier,
            RawToken::Nothing => TokenType::Nothing,
        }
    }

    pub fn is_synchronizer(&self) -> bool {
        use TokenType::*;
        matches!(
            self.get_type(),
            Int | Void
        )
    }

    pub fn take(&mut self) -> Self {
        std::mem::take(self)
    }
}

impl Default for Token {
    fn default() -> Self {
        Self {
            inner: RawToken::Nothing,
            span: span!(0, 0),
        }
    }
}

impl RawToken {
    pub fn as_integer(self) -> i64 {
        if let RawToken::Integer(val) = self {
            val
        } else {
            panic!("Internal error: expected an integer token, found {:?}", self);
        }
    }

    pub fn as_identifier(self) -> String {
        if let RawToken::Identifier(val) = self {
            val
        } else {
            panic!("Internal error: expected an identifier token, found {:?}", self);
        }
    }
}