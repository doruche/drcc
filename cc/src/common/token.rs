//! Token, and tokens' types used in relevant stages of the compiler.

use std::fmt::Display;

use crate::{common::{span::Span, StrDescriptor}, span};
use crate::ast::AstBinaryOp;

/// Token without any additional information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RawToken {
    // no-lexeme tokens
    LParen, RParen,
    LBrace, RBrace,
    Tilde, Hyphen, DoubleHyphen,
    Plus, DoublePlus,
    Asterisk, ForwardSlash, Percent,
    Semicolon, Bang, And, Equal,
    LessThan, GreaterThan, NotEqual,
    GtEq, LtEq, Or, DoubleOr, DoubleAnd, DoubleEqual,
    Return, If, Else, QuestionMark, Colon,
    Int, Void, 


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
    Bang,
    And,
    DoubleAnd,
    Or,
    DoubleOr,
    Equal,
    DoubleEqual,
    LessThan,
    GreaterThan,
    GtEq,
    LtEq,
    NotEqual,
    Return,
    If,
    Else,
    QuestionMark,
    Colon,
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
            RawToken::Bang => TokenType::Bang,
            RawToken::And => TokenType::And,
            RawToken::Equal => TokenType::Equal,
            RawToken::LessThan => TokenType::LessThan,
            RawToken::GreaterThan => TokenType::GreaterThan,
            RawToken::GtEq => TokenType::GtEq,
            RawToken::LtEq => TokenType::LtEq,
            RawToken::Or => TokenType::Or,
            RawToken::DoubleOr => TokenType::DoubleOr,
            RawToken::DoubleAnd => TokenType::DoubleAnd,
            RawToken::DoubleEqual => TokenType::DoubleEqual,
            RawToken::NotEqual => TokenType::NotEqual,
            RawToken::Return => TokenType::Return,
            RawToken::If => TokenType::If,
            RawToken::Else => TokenType::Else,
            RawToken::QuestionMark => TokenType::QuestionMark,
            RawToken::Colon => TokenType::Colon,
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

    pub fn is_type(&self) -> bool {
        use TokenType::*;
        matches!(self.get_type(), Int | Void)
    }
 
    pub fn is_binary_op(&self) -> bool {
        use TokenType::*;
        matches!(
            self.get_type(),
            Plus | Hyphen | Asterisk | ForwardSlash | Percent |
            LessThan | GreaterThan | GtEq | LtEq | Equal | NotEqual |
            And | Or | DoubleAnd | DoubleOr | DoubleEqual
            // Fake binary operators
            | QuestionMark

        )
    }

    pub fn to_binary_op(&self) -> AstBinaryOp {
        use TokenType::*;
        match self.get_type() {
            Plus => AstBinaryOp::Add,
            Hyphen => AstBinaryOp::Sub,
            Asterisk => AstBinaryOp::Mul,
            ForwardSlash => AstBinaryOp::Div,
            Percent => AstBinaryOp::Rem,
            LessThan => AstBinaryOp::LessThan,
            GreaterThan => AstBinaryOp::GreaterThan,
            GtEq => AstBinaryOp::GtEq,
            LtEq => AstBinaryOp::LtEq,
            NotEqual => AstBinaryOp::NotEqual,
            DoubleAnd => AstBinaryOp::And,
            DoubleOr => AstBinaryOp::Or,
            DoubleEqual => AstBinaryOp::Equal,
            Equal => AstBinaryOp::Assign,
            QuestionMark => AstBinaryOp::Ternary,
            // And => AstBinaryOp::BitwiseAnd,
            // Or => AstBinaryOp::BitwiseOr,
            // Equal => AstBinaryOp::Assign,
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

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.inner)
    }
}
