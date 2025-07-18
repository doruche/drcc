use std::collections::HashMap;

use crate::common::*;
use crate::sem::{
    FuncSymbol, HirBinaryOp, HirParam, HirUnaryOp, StaticVarSymbol
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
    Var {
        name: StrDescriptor,
        local_id: Option<usize>,
    },
    Temp(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Param {
    pub name: StrDescriptor,
    pub data_type: DataType,
}

impl From<HirParam> for Param {
    fn from(param: HirParam) -> Self {
        Param {
            name: param.name,
            data_type: param.data_type,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LabelOperand {
    AutoGen(AutoGenLabel),
    Named {
        name: StrDescriptor,
        id: usize,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutoGenLabel {
    Branch(usize),  // normal auto-generated label for branches and loops
    Continue(usize),
    Break(usize),
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
    FuncCall {
        target: StrDescriptor,
        args: Vec<Operand>,
        dst: Operand,
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
    pub return_type: DataType,
    pub linkage: Linkage,
    pub name: StrDescriptor,
    pub params: Vec<Param>,
    pub body: Vec<Insn>,
}

#[derive(Debug, Clone)]
pub struct StaticVar {
    pub name: StrDescriptor,
    pub data_type: DataType,
    pub initializer: InitVal,
    pub linkage: Linkage,
}

#[derive(Debug)]
pub struct TopLevel {
    pub functions: HashMap<StrDescriptor, Function>,
    pub static_vars: HashMap<StrDescriptor, StaticVar>,
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