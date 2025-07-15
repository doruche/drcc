//! HIR representation

use crate::common::*;

#[derive(Debug)]
pub struct TopLevel {
    pub decls: Vec<Decl>,
    pub strtb: StringPool,
}

#[derive(Debug)]
pub enum BlockItem {
    Declaration(Decl),
    Statement(Stmt),
}

#[derive(Debug)]
pub enum Decl {
    FuncDecl {
        return_type: (DataType, Span),
        name: (StrDescriptor, Span),
        // params,
        body: Vec<BlockItem>,
    },
    VarDecl {
        name: (StrDescriptor, Span),
        data_type: (DataType, Span),
        initializer: Option<Box<TypedExpr>>,
        // initial value,
    }
}

#[derive(Debug)]
pub struct TypedExpr {
    pub expr: Expr,
    pub type_: DataType,
}

impl TypedExpr {
    pub fn untyped(expr: Expr) -> Self {
        Self {
            expr,
            type_: DataType::Indeterminate,
        }
    }
}

#[derive(Debug)]
pub enum Expr {
    IntegerLiteral(i64),
    Variable(StrDescriptor, Span),
    Assignment {
        span: Span,
        left: Box<TypedExpr>,
        right: Box<TypedExpr>,
    },
    Ternary {
        condition: Box<TypedExpr>,
        then_expr: Box<TypedExpr>,
        else_expr: Box<TypedExpr>,
    },
    Group(Box<TypedExpr>),
    Unary((UnaryOp, Span), Box<TypedExpr>),
    Binary {
        op: (BinaryOp, Span),
        left: Box<TypedExpr>,
        right: Box<TypedExpr>,
    },
}

#[derive(Debug)]
pub enum Stmt {
    Return {
        span: Span,
        expr: Box<TypedExpr>,
    },
    Expr(Box<TypedExpr>),
    If {
        condition: Box<TypedExpr>,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
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
    Ls,
    Gt,
    GtEq,
    LsEq,
    Eq,
    NotEq,
    And,
    Or,
    Assign,
}

use crate::ast::{AstUnaryOp, AstBinaryOp};

impl From<AstUnaryOp> for UnaryOp {
    fn from(op: AstUnaryOp) -> Self {
        match op {
            AstUnaryOp::Pos => UnaryOp::Pos,
            AstUnaryOp::Negate => UnaryOp::Negate,
            AstUnaryOp::Not => UnaryOp::Not,
            AstUnaryOp::Complement => UnaryOp::Complement,
        }
    }
}

impl From<AstBinaryOp> for BinaryOp {
    fn from(op: AstBinaryOp) -> Self {
        match op {
            AstBinaryOp::Add => BinaryOp::Add,
            AstBinaryOp::Sub => BinaryOp::Sub,
            AstBinaryOp::Mul => BinaryOp::Mul,
            AstBinaryOp::Div => BinaryOp::Div,
            AstBinaryOp::Rem => BinaryOp::Rem,
            AstBinaryOp::LessThan => BinaryOp::Ls,
            AstBinaryOp::GreaterThan => BinaryOp::Gt,
            AstBinaryOp::GtEq => BinaryOp::GtEq,
            AstBinaryOp::LtEq => BinaryOp::LsEq,
            AstBinaryOp::Equal => BinaryOp::Eq,
            AstBinaryOp::NotEqual => BinaryOp::NotEq,
            AstBinaryOp::And => BinaryOp::And,
            AstBinaryOp::Or => BinaryOp::Or,

            AstBinaryOp::Assign|
            AstBinaryOp::Ternary => unreachable!(),
        }
    }
}