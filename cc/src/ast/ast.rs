use crate::common::{
    DataType, 
    Span, 
    StrDescriptor, 
    StringPool,
};

#[derive(Debug)]
pub struct TopLevel {
    pub decls: Vec<Decl>,
    pub strtb: StringPool,
}

#[derive(Debug, Clone)]
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
        initializer: Option<Box<Expr>>,
        // initial value,
    }
}

#[derive(Debug, Clone)]
pub enum BlockItem {
    Declaration(Decl),
    Statement(Stmt),
}


#[derive(Debug, Clone)]
pub enum Expr {
    IntegerLiteral(i64, Span),
    Variable(StrDescriptor, Span),
    Assignment {
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Group(Box<Expr>),
    Unary((UnaryOp, Span), Box<Expr>),
    Binary {
        op: (BinaryOp, Span),
        left: Box<Expr>,
        right: Box<Expr>
    }
}

#[derive(Debug, Clone)]
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
    Complement,
    Not,
}

#[derive(Debug, Clone, Copy)]
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

impl BinaryOp {
    pub fn precedence(&self) -> u8 {
        match self {
            BinaryOp::Assign => 0,
            BinaryOp::Or => 2,
            BinaryOp::And => 3,
            BinaryOp::Equal | BinaryOp::NotEqual => 4,
            BinaryOp::LessThan | BinaryOp::GreaterThan | BinaryOp::GtEq | BinaryOp::LtEq => 5,
            BinaryOp::Add | BinaryOp::Sub => 6,
            BinaryOp::Mul | BinaryOp::Div | BinaryOp::Rem => 7,
        }
    }

    pub fn prior_than(&self, other: &BinaryOp) -> bool {
        self.precedence() > other.precedence()
    }
}