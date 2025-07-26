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
};

mod rig;
mod alloc;
mod live_analysis;

/// Register interference graph, per function.
#[derive(Debug)]
pub struct Rig {
    pub nodes: HashMap<GeneralReg, RigNode>,
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

#[derive(Debug)]
pub struct AnalyzeResult {
    pub map: HashMap<GeneralReg, Register>,
}