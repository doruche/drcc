use std::collections::HashMap;

use crate::common::*;
use crate::sem::{
    HirBinaryOp, HirParam, HirUnaryOp,
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Operand {
    Imm(Constant),
    Var {
        name: StrDescriptor,
        // Some(..) for local variables, None for static variables
        local_id: Option<usize>,
        data_type: DataType,
    },
    Temp(usize, DataType),
}

impl Operand {
    pub fn data_type(&self) -> DataType {
        match self {
            Operand::Imm(constant) => constant.data_type(),
            Operand::Var { data_type, .. } => *data_type,
            Operand::Temp(_, data_type) => *data_type,
        }
    }

    pub fn is_static(&self) -> bool {
        match self {
            Operand::Imm(_) => false,
            Operand::Var { local_id: None, .. } => true,
            Operand::Var { local_id: Some(_), .. } => false,
            Operand::Temp(_, _) => false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Param {
    pub name: StrDescriptor,
    pub data_type: DataType,
    pub local_id: usize,
}

impl From<HirParam> for Param {
    fn from(param: HirParam) -> Self {
        Param {
            name: param.name,
            data_type: param.data_type,
            local_id: param.local_id,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LabelOperand {
    AutoGen(AutoGenLabel),
    Named {
        name: StrDescriptor,
        id: usize,
    },
}

impl LabelOperand {
    pub fn id(&self) -> usize {
        match self {
            LabelOperand::AutoGen(auto_gen) => match auto_gen {
                AutoGenLabel::Branch(id) => *id,
                AutoGenLabel::Continue(id) => *id,
                AutoGenLabel::Break(id) => *id,
            },
            LabelOperand::Named { id, .. } => *id,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AutoGenLabel {
    Branch(usize),  // normal auto-generated label for branches and loops
    Continue(usize),
    Break(usize),
}

#[derive(Debug, Clone)]
pub enum Insn {
    Return(Operand),
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
    Truncate {
        src: Operand,
        dst: Operand,
    },
    SignExt {
        src: Operand,
        dst: Operand,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocalVar {
    pub name: StrDescriptor,
    pub local_id: usize,
    pub data_type: DataType,
}

#[derive(Debug, Clone)]
pub enum Function {
    Defined {
        return_type: DataType,
        linkage: Linkage,
        name: StrDescriptor,
        params: Vec<Param>,
        local_vars: HashMap<usize, LocalVar>,
        body: Vec<Insn>,
    },
    Declared {
        linkage: Linkage,
        name: StrDescriptor,
        type_: FuncType,
    }
}

impl Function {
    pub fn name(&self) -> StrDescriptor {
        match self {
            Function::Defined { name, .. } => *name,
            Function::Declared { name, .. } => *name,
        }
    }

    pub fn type_(&self) -> FuncType {
        match self {
            Function::Defined { return_type, params, .. } => FuncType {
                return_type: *return_type,
                param_types: params.iter().map(|p| p.data_type).collect(),
            },
            Function::Declared { type_, .. } => type_.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct StaticVar {
    pub name: StrDescriptor,
    pub data_type: DataType,
    pub initializer: InitVal,
    pub linkage: Linkage,
}

#[derive(Debug, Clone)]
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
            
            HirBinaryOp::Assign => unreachable!(),
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