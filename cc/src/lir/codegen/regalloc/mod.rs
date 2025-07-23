//! 2nd pass of LIR code generation.
//! Map virtual registers to physical registers or spill them to memory.

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

/// Register interference graph, per function.
#[derive(Debug)]
pub struct Rig {
    pub nodes: HashMap<RigNode, Vec<RigNode>>,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RigNode {
    pub reg: GeneralReg,
    pub spill_cost: usize,
    pub color: Option<Register>,
}

impl RigNode {
    pub fn new(reg: GeneralReg, spill_cost: usize) -> Self {
        RigNode {
            reg,
            spill_cost,
            color: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GeneralReg {
    Phys(Register),
    Virt(usize),
}