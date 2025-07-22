
use std::collections::HashMap;

use crate::{asm::Register, common::*};
use super::{
    Rig,
    RigNode,
    GeneralReg,
    TopLevel,
    Function,
    Insn,
    Operand,
};

impl Rig {
    pub fn build(function: &Function) -> Self {
        // We'll split the procedure into several parts:
        // 1. Build a base interference graph with all physical registers.
        // 2. Add virtual registers into the graph.
        // 3. Liveness analysis, with a control flow graph.
        // 4. Add edges between virtual registers.

        let mut rig = Self::base();
        rig.add_virtreg(function);

        rig
    }

    fn add_virtreg(&mut self, function: &Function) {
        todo!()
    }

    fn base() -> Self {
        // build a base interference graph
        // we won't use following registers:
        // - zero, ra, sp, gp, tp, s0/fp, cz these serve special purposes
        // - t5, t6, cz we'll keep them as our scratch registers when loading and storing virtual registers
        let mut nodes = HashMap::new();

        for i in 0..32 {
            if matches!(i, 0..=4|8|30|31) {
                continue;
            }
            let node = RigNode::new(GeneralReg::Phys(Register::x(i)), 0);
            nodes.insert(node, vec![]);
        }

        // adding edge between physical registers
        let keys: Vec<_> = nodes.keys().cloned().collect();
        for (node, edges) in nodes.iter_mut() {
            for other in &keys {
                if node.reg != other.reg {
                    edges.push(*other);
                }
            }
        }

        Self {
            nodes,
        }
    }
}