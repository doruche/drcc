use crate::common::*;
use crate::sem::{
    HirBinaryOp,
    HirUnaryOp,
};


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Pos,
    Negate,
    Complement,
    Not,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operand {
    Imm(i64),
    Var(StrDescriptor),
    Temp(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LabelOperand {
    AutoGen(usize),
    Named {
        name: StrDescriptor,
        id: usize,
    },
}

#[derive(Debug, Clone)]
pub enum Insn {
    Return(Option<Operand>),
    Unary {
        op: UnaryOp,
        src: Operand,
        dst: Operand,
    },
    Binary {
        op: BinaryOp,
        left: Operand,
        right: Operand,
        dst: Operand,
    },
    Label(LabelOperand),
    Jump(LabelOperand),
    BranchIfZero {
        src: Operand,
        label: LabelOperand,
    },
    BranchNotZero {
        src: Operand,
        label: LabelOperand,
    },
    Move {
        src: Operand,
        dst: Operand,
    },
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: StrDescriptor,
    // pub params,
    pub return_type: DataType,
    pub body: Vec<Insn>,
}

#[derive(Debug)]
pub struct TopLevel {
    pub functions: Vec<Function>,
    // global variables,
    pub strtb: StringPool,
}

impl From<HirBinaryOp> for BinaryOp {
    fn from(op: HirBinaryOp) -> Self {
        match op {
            HirBinaryOp::Add => BinaryOp::Add,
            HirBinaryOp::Sub => BinaryOp::Sub,
            HirBinaryOp::Mul => BinaryOp::Mul,
            HirBinaryOp::Div => BinaryOp::Div,
            HirBinaryOp::Rem => BinaryOp::Rem,
            HirBinaryOp::Ls => BinaryOp::Ls,
            HirBinaryOp::Gt => BinaryOp::Gt,
            HirBinaryOp::GtEq => BinaryOp::GtEq,
            HirBinaryOp::LsEq => BinaryOp::LsEq,
            HirBinaryOp::Eq => BinaryOp::Eq,
            HirBinaryOp::NotEq => BinaryOp::NotEq,
            HirBinaryOp::And => BinaryOp::And,
            HirBinaryOp::Or => BinaryOp::Or,
            HirBinaryOp::Assign => BinaryOp::Assign,
        }
    }
}

impl From<HirUnaryOp> for UnaryOp {
    fn from(op: HirUnaryOp) -> Self {
        match op {
            HirUnaryOp::Pos => UnaryOp::Pos,
            HirUnaryOp::Negate => UnaryOp::Negate,
            HirUnaryOp::Complement => UnaryOp::Complement,
            HirUnaryOp::Not => UnaryOp::Not,
        }
    }
}