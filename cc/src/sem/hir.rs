//! HIR representation

use std::collections::HashMap;

use crate::common::*;

use crate::ast::AstParam;

#[derive(Debug)]
pub struct TopLevel {
    pub strtb: StringPool,
    pub funcs: HashMap<StrDescriptor, Function>,
    pub static_vars: HashMap<StrDescriptor, StaticVar>,
}

// Debug
impl TopLevel {
    pub fn dump_funcs(&self) -> String {
        let mut output = String::new();
        for (name, func) in &self.funcs {
            output.push_str(&format!("[{}] fn {}(", func.linkage, self.strtb.get(*name).unwrap()));
            let mut params = func.params.iter();
            if params.len() == 0 {
                output.push_str("void");
            } else {
                let first = params.next().unwrap();
                output.push_str(&format!("{} {}", 
                    self.strtb.get(first.name).unwrap(),
                    first.data_type,
                ));
                for param in params {
                    output.push_str(&format!(", {} {}", 
                        self.strtb.get(param.name).unwrap(),
                        param.data_type,
                    ));
                }
            }
            output.push_str(&format!(") -> {}", func.return_type));
            output.push_str(&format!(" {}\n",
                if let Some(body) = func.body.as_ref() {
                    format!("{:#?}", body)
                } else {
                    "".to_string()
                },
            ));
        }
        output
    }

    pub fn dump_static_vars(&self) -> String {
        let mut output = String::new();
        for (name, var) in &self.static_vars {
            output.push_str(&format!("[{}]\n{} {} ",
            var.linkage,
            var.data_type,
            self.strtb.get(*name).unwrap(), 
            ));
            match &var.initializer {
                InitVal::None => output.push_str("= undefined\n"),
                InitVal::Const(c) => output.push_str(&format!("= {:?}\n", c)),
                InitVal::Tentative => output.push_str("= tentative\n"),
            }
        }
        output
    }
}

#[derive(Debug, Clone)]
pub enum BlockItem {
    Declaration(LocalVarDecl),
    Statement(Stmt),
}

#[derive(Debug, Clone)]
pub struct LocalVarDecl {
    pub name: StrDescriptor,
    pub data_type: DataType,
    pub local_id: usize,
    pub span: Span,
    pub initializer: Option<TypedExpr>,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: StrDescriptor,

    // 'params' field will be the parameters of the definition of the function if there exists one,
    // otherwise just a blank vector.
    // this is because we need names of parameters to generate code.
    // but if there does not exist a definition, names do not matter.
    pub params: Vec<Param>,
    
    pub return_type: DataType,
    pub linkage: Linkage,
    pub body: Option<Vec<BlockItem>>,
}

#[derive(Debug, Clone)]
pub struct StaticVar {
    pub name: StrDescriptor,
    pub data_type: DataType,
    pub linkage: Linkage,
    pub initializer: InitVal,
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: StrDescriptor,
    pub data_type: DataType,
    pub local_id: usize,
}

impl Param {
    pub fn type_eq(&self, other: &Param) -> bool {
        self.data_type == other.data_type
    }
}


#[derive(Debug, Clone)]
pub struct TypedExpr {
    pub untyped: Expr,
    pub type_: DataType,
}

impl TypedExpr {
    pub fn untyped(expr: Expr) -> Self {
        Self {
            untyped: expr,
            type_: DataType::Indeterminate,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Variable {
    Local {
        name: StrDescriptor,
        // every function has its own local variable id counter.
        // we use this field to distinguish local variables.
        local_id: usize,
        data_type: DataType,
    },
    Static {
        name: StrDescriptor,
        data_type: DataType,
    },
}

impl Variable {
    pub fn data_type(&self) -> DataType {
        match self {
            Variable::Local { data_type, .. } => *data_type,
            Variable::Static { data_type, .. } => *data_type,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Expr {
    IntegerLiteral(Constant),
    Var(Variable),
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
        span: Span,
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
    Cast {
        target: DataType,
        expr: Box<TypedExpr>,
        span: Span,
    }
}


#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub enum ForInit {
    Declaration(LocalVarDecl),
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

impl BinaryOp {
    pub fn is_arithmetic(&self) -> bool {
        matches!(self, BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Rem)
    }
}