use std::collections::{HashMap, HashSet};

use crate::{asm::Register, common::*};
use super::{
    CodeGen,
    RegAlloc,
    Canonic,
    Spill,
    TopLevel,
    Function,
    Insn,
    Operand,
    FuncContext,
};

mod rig;
mod alloc;
mod live_analysis;

/// Register interference graph, per function.
#[derive(Debug)]
pub struct Rig<'a> {
    pub nodes: HashMap<GeneralReg, RigNode>,
    pub func_cxs: &'a HashMap<StrDescriptor, FuncContext>,
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RigNode {
    pub reg: GeneralReg,
    pub spill_cost: usize,
    pub color: Option<Register>,
    pub neighbors: HashSet<GeneralReg>,
}

impl RigNode {
    pub fn new(reg: GeneralReg, spill_cost: usize) -> Self {
        RigNode {
            reg,
            spill_cost,
            color: None,
            neighbors: HashSet::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GeneralReg {
    Phys(Register),
    Virt(usize),
}

impl TryFrom<Operand> for GeneralReg {
    type Error = ();

    fn try_from(op: Operand) -> std::result::Result<Self, Self::Error> {
        match op {
            Operand::PhysReg(reg) => Ok(GeneralReg::Phys(reg)),
            Operand::VirtReg(virt_reg) => Ok(GeneralReg::Virt(virt_reg)),
            _ => Err(()),
        }
    }
}

#[derive(Debug)]
pub struct AnalyzeResult {
    pub map: HashMap<GeneralReg, Option<Register>>,
}