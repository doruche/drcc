use crate::{asm::Register, common::*, tac::LabelOperand};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operand {
    PseudoReg(usize),
    PhysReg(Register),
    Imm(i64),
    StackSlot(i32), // relative to fp/s8
    Static(StrDescriptor),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Insn {
    Add(Operand, Operand, Operand),
    Addi(Operand, Operand, Operand),
    Sub(Operand, Operand, Operand),
    Mul(Operand, Operand, Operand),
    Div(Operand, Operand, Operand),
    Slt(Operand, Operand, Operand),
    Jmp(LabelOperand),
    Beq(Operand, Operand, LabelOperand),
    Bne(Operand, Operand, LabelOperand),
    Call(StrDescriptor),
    Ret,
    Lw(Operand, Operand),
    Sw(Operand, Operand),
    Li(Operand, i64),
    La(Operand, StrDescriptor),
    Neg(Operand, Operand),
    Not(Operand, Operand),
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: StrDescriptor,
    pub linkage: Linkage,
    pub body: Vec<Insn>,
    pub stack_size: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StaticVar {
    pub name: StrDescriptor,
    pub size: usize,
    pub linkage: Linkage,
    pub initializer: Option<Constant>,
}

#[derive(Debug, Clone)]
pub struct DataSegment {
    pub items: Vec<StaticVar>,
}

#[derive(Debug, Clone)]
pub struct BssSegment {
    pub items: Vec<StaticVar>,
}

#[derive(Debug, Clone)]
pub struct TopLevel {
    pub functions: Vec<Function>,
    pub data_seg: Option<DataSegment>,
    pub bss_seg: Option<BssSegment>,
}