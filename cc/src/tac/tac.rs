use crate::common::{DataType, StrDescriptor, StringPool};

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
    Label(usize),
    Jump(usize),
    BranchIfZero {
        src: Operand,
        label: usize,
    },
    BranchNotZero {
        src: Operand,
        label: usize,
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

#[derive(Debug, Clone)]
pub struct TopLevel {
    pub functions: Vec<Function>,
}

impl TopLevel {
    fn dump(&self, strtb: &StringPool) -> String {
        let insns = format!("{:#}", self);
        let strtb = strtb.dump();
        format!("{}\nString Table:\n{}", insns, strtb)
    }
}