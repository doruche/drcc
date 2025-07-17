//! HIR representation

use std::collections::HashMap;

use crate::common::*;

use crate::ast::AstParam;

#[derive(Debug)]
pub struct TopLevel {
    pub decls: Vec<Decl>,
    pub strtb: StringPool,
    pub funcs: HashMap<StrDescriptor, FuncSymbol>,
    pub static_vars: HashMap<StrDescriptor, StaticVarSymbol>,
}

// Debug
impl TopLevel {
    pub fn dump_funcs(&self) -> String {
        let mut output = String::new();
        for (name, func) in &self.funcs {
            output.push_str(&format!("[{}] fn {}(", func.linkage, self.strtb.get(*name).unwrap()));
            let mut params = func.type_.param_types.iter();
            if params.len() == 0 {
                output.push_str("void");
            } else {
                output.push_str(&format!("{}", params.next().unwrap()));
                for param in params {
                    output.push_str(&format!(", {}", param));
                }
            }
            output.push_str(&format!(") -> {}\n", func.type_.return_type));
        }
        output
    }
}

#[derive(Debug)]
pub enum BlockItem {
    Declaration(Decl),
    Statement(Stmt),
}


#[derive(Debug)]
pub enum Decl {
    // when tac codegen is implemented, non-automatic symbols will be
    // retrieved from the symbol table.
    // so there is no need to store too much information here.

    FuncDecl {
        name: (StrDescriptor, Span),
        params: Vec<Param>,
        body: Option<Vec<BlockItem>>,
    },
    VarDecl {
        name: (StrDescriptor, Span),
        data_type: (DataType, Span),
        storage_class: Option<(StorageClass, Span)>,
        initializer: Option<Box<TypedExpr>>,
    }
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: StrDescriptor,
    pub data_type: DataType,
    pub span: Span,
}

impl Param {
    pub fn type_eq(&self, other: &Param) -> bool {
        self.data_type == other.data_type
    }
}

impl From<AstParam> for Param {
    fn from(value: AstParam) -> Self {
        Self {
            name: value.name,
            data_type: value.data_type,
            span: value.span,
        }
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
    FuncCall {
        name: StrDescriptor,
        span: Span,
        args: Vec<TypedExpr>,
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
    Break {
        span: Span,
        loop_label: usize,
    },
    Continue {
        span: Span,
        loop_label: usize,
    },
    While {
        span: Span,
        controller: Box<TypedExpr>,
        body: Box<Stmt>,
        loop_label: usize,
    },
    DoWhile {
        span: Span,
        body: Box<Stmt>,
        controller: Box<TypedExpr>,
        loop_label: usize,
    },
    For {
        span: Span,
        initializer: Option<Box<ForInit>>,
        controller: Option<Box<TypedExpr>>,
        post: Option<Box<TypedExpr>>,
        body: Box<Stmt>,
        loop_label: usize,
    },
    Compound(Vec<BlockItem>),
    Nil,
}

#[derive(Debug)]
pub enum ForInit {
    Declaration(Decl),
    Expression(Box<TypedExpr>),
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
use crate::sem::symtb::StaticVarSymbol;
use crate::sem::FuncSymbol;

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