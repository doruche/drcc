//! HIR representation

use crate::common::*;
use super::{
    SymbolId,
    SymbolTable,
};

#[derive(Debug)]
pub struct TopLevel {
    pub decls: Vec<Decl>,
    pub strtb: StringPool,
    pub symtb: SymbolTable,
}

#[derive(Debug)]
pub enum BlockItem {
    Declaration(Decl),
    Statement(Stmt),
}

#[derive(Debug)]
pub enum Decl {
    FuncDecl {
        id: SymbolId,       
        return_type: (DataType, Span),
        name: (StrDescriptor, Span),
        // params,
        body: Vec<BlockItem>,
    },
    VarDecl {
        id: SymbolId,
        name: (StrDescriptor, Span),
        data_type: (DataType, Span),
        initializer: Option<Box<Expr>>,
        // initial value,
    }
}

#[derive(Debug)]
pub enum Expr {
    IntegerLiteral(i64, Span),
    Variable(SymbolId, StrDescriptor, Span),
    Assignment {
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Group(Box<Expr>),
    Unary((UnaryOp, Span), Box<Expr>),
    Binary {
        op: (BinaryOp, Span),
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Nil,
}

#[derive(Debug)]
pub enum Stmt {
    Return {
        span: Span,
        expr: Box<Expr>,
    },
    Expr(Box<Expr>),
    Nil,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Pos,
    Negate,
    Not,
    Complement,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    LessThan,
    GreaterThan,
    GtEq,
    LtEq,
    Equal,
    NotEqual,
    And,
    Or,
    Assign,
}