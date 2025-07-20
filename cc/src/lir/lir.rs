use std::collections::HashMap;

use crate::{asm::Register, common::*, tac::LabelOperand};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operand {
    VirtReg(usize),
    PhysReg(Register),
    Imm(i64),
    Frame(i32), // relative to fp/s8
    Static(StrDescriptor),
}

/// Some instructions' usage may overlap with others. (e.g. addi t0, t1, 0 vs. mv t0, t1)
/// We use such pseudo-instructions to emit more readable code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Insn {
    Add(Operand, Operand, Operand),
    Addi(Operand, Operand, Operand),
    Sub(Operand, Operand, Operand),
    Slt(Operand, Operand, Operand),
    Jmp(LabelOperand),
    Beq(Operand, Operand, LabelOperand),
    Bne(Operand, Operand, LabelOperand),
    Call(StrDescriptor),
    Ret,
    Lw(Operand, Operand),
    Sw(Operand, Operand),
    Ld(Operand, Operand),
    Sd(Operand, Operand),
    Ldu(Operand, Operand),
    Mv(Operand, Operand),
    Li(Operand, i64),
    La(Operand, StrDescriptor),
    Neg(Operand, Operand),
    Not(Operand, Operand),
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: StrDescriptor,
    pub linkage: Linkage,
    pub func_type: FuncType,
    pub body: Vec<Insn>,
    pub frame_size: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StaticVar {
    pub name: StrDescriptor,
    pub data_type: DataType,
    pub linkage: Linkage,
    pub initializer: InitVal,
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
    pub functions: HashMap<StrDescriptor, Function>,
    pub data_seg: DataSegment,
    pub bss_seg: BssSegment,
}