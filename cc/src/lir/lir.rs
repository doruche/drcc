use std::collections::HashMap;

use crate::{asm::Register, common::*, tac::{TacAutoGenLabel, TacLabelOperand}};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operand {
    VirtReg(usize),
    PhysReg(Register),
    Imm(i64),
    Frame(isize, usize), // relative to fp/s8
    Stack(isize, usize), // relative to sp
    Static(StrDescriptor),
}

/// Some instructions' usage may overlap with others. (e.g. addi t0, t1, 0 vs. mv t0, t1)
/// We use such pseudo-instructions to emit more readable code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Insn {
    Add(Operand, Operand, Operand),
    Addi(Operand, Operand, i64),
    Addw(Operand, Operand, Operand),
    Addiw(Operand, Operand, i32),
    Sub(Operand, Operand, Operand),
    Subw(Operand, Operand, Operand),
    Mul(Operand, Operand, Operand),
    Mulw(Operand, Operand, Operand),
    Div(Operand, Operand, Operand),
    Divw(Operand, Operand, Operand),
    Rem(Operand, Operand, Operand),
    Remw(Operand, Operand, Operand),
    Slt(Operand, Operand, Operand),
    Sgt(Operand, Operand, Operand),
    Seqz(Operand, Operand),
    Snez(Operand, Operand),
    Sextw(Operand, Operand),
    Label(LabelOperand),
    J(LabelOperand),
    Beq(Operand, Operand, LabelOperand),
    Bne(Operand, Operand, LabelOperand),
    Call(StrDescriptor),
    Ret,
    Lw(Operand, Operand),
    Sw(Operand, Operand),
    Ld(Operand, Operand),
    Sd(Operand, Operand),
    Mv(Operand, Operand),
    Li(Operand, i64),
    La(Operand, StrDescriptor),
    Neg(Operand, Operand),
    Negw(Operand, Operand),
    Not(Operand, Operand),

    LoadStatic(Operand, StrDescriptor),
    StoreStatic(StrDescriptor, Operand),

    Intermediate(IntermediateInsn),
}

/// Instructions that are used during the intermediate stages of code generation.
/// When lir is finally emitted, these instructions will be replaced with prologue/epilogue instructions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntermediateInsn {
    Prologue,
    Epilogue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LabelOperand {
    AutoGen(usize),
    Named(StrDescriptor),   
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LabelSignature {
    Tac {
        func: StrDescriptor,
        label: TacLabelOperand,
    },   
}

impl LabelSignature {
    pub fn from_tac(
        func: StrDescriptor,
        label: TacLabelOperand,
    ) -> Self {
        LabelSignature::Tac { func, label }
    }
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: StrDescriptor,
    pub linkage: Linkage,
    pub func_type: FuncType,
    pub body: Vec<Insn>,
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

#[derive(Debug)]
pub struct TopLevel {
    pub functions: HashMap<StrDescriptor, Function>,
    pub data_seg: DataSegment,
    pub bss_seg: BssSegment,
    pub strtb: StringPool,
}