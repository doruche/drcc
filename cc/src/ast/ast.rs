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
    IntegerLiteral(i64),
    Variable(StrDescriptor, Span),
    Assignment {
        span: Span,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Ternary {
        condition: Box<Expr>,
        then_expr: Box<Expr>,
        else_expr: Box<Expr>,
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
    If {
        condition: Box<Expr>,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    Compound(Vec<BlockItem>),
    Break(Span),
    Continue(Span),
    While {
        span: Span,
        controller: Box<Expr>,
        body: Box<Stmt>,
    },
    DoWhile {
        span: Span,
        body: Box<Stmt>,
        controller: Box<Expr>,
    },
    For {
        span: Span,
        initializer: Option<Box<ForInit>>,
        controller: Option<Box<Expr>>,
        post: Option<Box<Expr>>,
        body: Box<Stmt>,
    },
    Nil,
}

#[derive(Debug, Clone)]
pub enum ForInit {
    Declaration(Decl),
    Expression(Expr),
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
    Ternary,
}

impl BinaryOp {
    pub const MIN_PRECEDENCE: u8 = 0;

    pub fn precedence(&self) -> u8 {
        match self {
            BinaryOp::Assign => 0,
            BinaryOp::Ternary => 1,
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