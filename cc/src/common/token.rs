//! Token, and tokens' types used in relevant stages of the compiler.

use crate::{common::{span::Span, BinaryOp, StrDescriptor}, span};

/// Token without any additional information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RawToken {
    // no-lexeme tokens
    LParen, RParen,
    LBrace, RBrace,
    Tilde, Hyphen, DoubleHyphen,
    Plus, DoublePlus,
    Asterisk, ForwardSlash, Percent,
    Semicolon,
    Return, Int, Void,


    // [0-9]+
    Integer(i64),
    // [a-zA-Z_][a-zA-Z0-9_]*
    Identifier(StrDescriptor),

    Nothing,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenType {
    LParen,
    RParen,
    LBrace,
    RBrace,
    Tilde,
    Hyphen,
    Plus,
    DoublePlus,
    DoubleHyphen,
    Asterisk,
    ForwardSlash,
    Percent,
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
            RawToken::Tilde => TokenType::Tilde,
            RawToken::Hyphen => TokenType::Hyphen,
            RawToken::DoubleHyphen => TokenType::DoubleHyphen,
            RawToken::Plus => TokenType::Plus,
            RawToken::DoublePlus => TokenType::DoublePlus,
            RawToken::Asterisk => TokenType::Asterisk,
            RawToken::ForwardSlash => TokenType::ForwardSlash,
            RawToken::Percent => TokenType::Percent,
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

    pub fn is_binary_op(&self) -> bool {
        use TokenType::*;
        matches!(
            self.get_type(),
            Plus | Hyphen | Asterisk | ForwardSlash | Percent
        )
    }

    pub fn to_binary_op(&self) -> BinaryOp {
        use TokenType::*;
        match self.get_type() {
            Plus => BinaryOp::Add,
            Hyphen => BinaryOp::Sub,
            Asterisk => BinaryOp::Mul,
            ForwardSlash => BinaryOp::Div,
            Percent => BinaryOp::Rem,
            _ => panic!("Internal error: expected a binary operator token, found {:?}", self),
        }
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

    pub fn as_identifier(self) -> StrDescriptor {
        if let RawToken::Identifier(val) = self {
            val
        } else {
            panic!("Internal error: expected an identifier token, found {:?}", self);
        }
    }
}