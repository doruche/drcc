use std::collections::HashMap;
use std::fmt::Display;
use crate::common::*;
use crate::lir::{
    LirFunction,
    LirBssSegment,
    LirDataSegment,
    LirStaticVar,
    LirLabelOperand,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Register {
    Zero, Ra, Sp, Gp, Tp,
    T0, T1, T2, S0, S1,
    A0, A1, A2, A3, A4,
    A5, A6, A7, S2, S3,
    S4, S5, S6, S7, S8,
    S9, S10, S11, T3, T4,
    T5, T6,
}

impl Display for Register {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Register::Zero => "zero",
            Register::Ra => "ra",
            Register::Sp => "sp",
            Register::Gp => "gp",
            Register::Tp => "tp",
            Register::T0 => "t0",
            Register::T1 => "t1",
            Register::T2 => "t2",
            Register::S0 => "s0",
            Register::S1 => "s1",
            Register::A0 => "a0",
            Register::A1 => "a1",
            Register::A2 => "a2",
            Register::A3 => "a3",
            Register::A4 => "a4",
            Register::A5 => "a5",
            Register::A6 => "a6",
            Register::A7 => "a7",
            Register::S2 => "s2",
            Register::S3 => "s3",
            Register::S4 => "s4",
            Register::S5 => "s5",
            Register::S6 => "s6",
            Register::S7 => "s7",
            Register::S8 => "s8",
            Register::S9 => "s9",
            Register::S10 => "s10",
            Register::S11 => "s11",
            Register::T3 => "t3",
            Register::T4 => "t4",
            Register::T5 => "t5",
            Register::T6 => "t6",
        };
        write!(f, "{name}")
    }
}

#[derive(Debug, Clone)]
pub struct RegIter {
    pub regs: [Register; 32],
    pub index: usize,
}

impl Iterator for RegIter {
    type Item = Register;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.regs.len() {
            let reg = self.regs[self.index];
            self.index += 1;
            Some(reg)
        } else {
            None
        }
    }
}

impl Register {
    pub const ALLOC_REGS: [&Self; 24] = [
        &Register::A0, &Register::A1, &Register::A2, &Register::A3,
        &Register::A4, &Register::A5, &Register::A6, &Register::A7,
        &Register::T0, &Register::T1, &Register::T2, &Register::T3,
        &Register::T4, &Register::S1, &Register::S2, &Register::S3, 
        &Register::S4, &Register::S5, &Register::S6, &Register::S7, 
        &Register::S8, &Register::S9, &Register::S10, &Register::S11,
    ];

    pub const fn iter() -> RegIter {
        RegIter {
            regs: [
                Register::Zero, Register::Ra, Register::Sp, Register::Gp, Register::Tp,
                Register::T0, Register::T1, Register::T2, Register::S0, Register::S1,
                Register::A0, Register::A1, Register::A2, Register::A3, Register::A4,
                Register::A5, Register::A6, Register::A7, Register::S2, Register::S3,
                Register::S4, Register::S5, Register::S6, Register::S7, Register::S8,
                Register::S9, Register::S10, Register::S11, Register::T3, Register::T4,
                Register::T5, Register::T6,
            ],
            index: 0,
        }
    }

    pub fn x(id: usize) -> Self {
        match id {
            0 => Register::Zero,
            1 => Register::Ra,
            2 => Register::Sp,
            3 => Register::Gp,
            4 => Register::Tp,
            5 => Register::T0,
            6 => Register::T1,
            7 => Register::T2,
            8 => Register::S0,
            9 => Register::S1,
            10 => Register::A0,
            11 => Register::A1,
            12 => Register::A2,
            13 => Register::A3,
            14 => Register::A4,
            15 => Register::A5,
            16 => Register::A6,
            17 => Register::A7,
            18 => Register::S2,
            19 => Register::S3,
            20 => Register::S4,
            21 => Register::S5,
            22 => Register::S6,
            23 => Register::S7,
            24 => Register::S8,
            25 => Register::S9,
            26 => Register::S10,
            27 => Register::S11,
            28 => Register::T3,
            29 => Register::T4,
            30 => Register::T5,
            31 => Register::T6,
            _ => panic!("Internal error: Invalid register ID: {}", id),
        }
    }

    pub fn a(id: usize) -> Self {
        match id {
            0 => Register::A0,
            1 => Register::A1,
            2 => Register::A2,
            3 => Register::A3,
            4 => Register::A4,
            5 => Register::A5,
            6 => Register::A6,
            7 => Register::A7,
            _ => panic!("Internal error: Invalid A register ID: {}", id),
        }
    }

    pub fn is_caller_saved(&self) -> bool {
        matches!(self, 
            Register::T0 | Register::T1 | Register::T2 | 
            Register::T3 | Register::T4 | Register::T5 | 
            Register::T6 |
            Register::A0 | Register::A1 | Register::A2 |
            Register::A3 | Register::A4 | Register::A5 |
            Register::A6 | Register::A7 |
            Register::Ra
        )
    }

    pub fn is_callee_saved(&self) -> bool {
        matches!(self, 
            Register::S0 | Register::S1 | Register::S2 |
            Register::S3 | Register::S4 | Register::S5 |
            Register::S6 | Register::S7 | Register::S8 |
            Register::S9 | Register::S10 | Register::S11 |
            Register::Sp
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Insn {
    Add(Register, Register, Register),
    Addi(Register, Register, i64),
    Addw(Register, Register, Register),
    Addiw(Register, Register, i32),
    Sub(Register, Register, Register),
    Subw(Register, Register, Register),
    Mul(Register, Register, Register),
    Mulw(Register, Register, Register),
    Div(Register, Register, Register),
    Divw(Register, Register, Register),
    Rem(Register, Register, Register),
    Remw(Register, Register, Register),

    Neg(Register, Register),
    Negw(Register, Register),
    Not(Register, Register), 
    Seqz(Register, Register),
    Snez(Register, Register),
    Sextw(Register, Register),
    Mv(Register, Register),

    Call(StrDescriptor),
    Beq(Register, Register, LabelOperand),
    Bne(Register, Register, LabelOperand),
    J(LabelOperand),
    Label(LabelOperand),
    Ret,

    Slt(Register, Register, Register),
    Sgt(Register, Register, Register),


    Ld(Register, Register, isize),
    Lw(Register, Register, isize),
    Sd(Register, Register, isize),
    Sw(Register, Register, isize),

    Li(Register, i64),
    La(Register, StrDescriptor),

    LoadStatic(Register, StrDescriptor),
    StoreStatic(Register, StrDescriptor),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LabelOperand {
    AutoGen(usize),
    Named(StrDescriptor),
}

impl From<LirLabelOperand> for LabelOperand {
    fn from(label: LirLabelOperand) -> Self {
        match label {
            LirLabelOperand::AutoGen(id) => LabelOperand::AutoGen(id),
            LirLabelOperand::Named(name) => LabelOperand::Named(name),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StaticVar {
    pub name: StrDescriptor,
    pub data_type: DataType,
    pub linkage: Linkage,
    pub initializer: InitVal,
}

impl From<LirStaticVar> for StaticVar {
    fn from(var: LirStaticVar) -> Self {
        StaticVar {
            name: var.name,
            data_type: var.data_type,
            linkage: var.linkage,
            initializer: var.initializer,
        }
    }
}

#[derive(Debug)]
pub struct DataSegment {
    pub items: HashMap<StrDescriptor, StaticVar>,
}

#[derive(Debug)]
pub struct BssSegment {
    pub items: HashMap<StrDescriptor, StaticVar>,
}

impl DataSegment {
    pub fn new() -> Self {
        DataSegment { items: HashMap::new() }
    }

    pub fn add(&mut self, var: StaticVar) {
        self.items.insert(var.name, var);
    }
}

impl BssSegment {
    pub fn new() -> Self {
        BssSegment { items: HashMap::new() }
    }

    pub fn add(&mut self, var: StaticVar) {
        self.items.insert(var.name, var);
    }
}

#[derive(Debug)]
pub struct Function {
    pub name: StrDescriptor,
    pub func_type: FuncType,
    pub body: Vec<Insn>,
    pub linkage: Linkage,
}


#[derive(Debug)]
pub struct TopLevel {
    pub functions: HashMap<StrDescriptor, Function>,
    pub data_seg: DataSegment,
    pub bss_seg: BssSegment,
    pub strtb: StringPool,    
}