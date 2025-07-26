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
    AnalyzeResult,
};

impl Rig {
    pub fn analyze(self) -> AnalyzeResult {
        todo!()
    }

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

    fn base() -> Self {
        // build a base interference graph
        // we won't use following registers:
        // - zero, ra, sp, gp, tp, s0/fp, cz these serve special purposes
        // - t5, t6, cz we'll keep them as scratch registers when loading and storing virtual registers
        let mut nodes = HashMap::new();

        for i in 0..32 {
            if matches!(i, 0..=4|8|30|31) {
                continue;
            }
            let reg = GeneralReg::Phys(Register::x(i));
            let node = RigNode::new(reg, 0);
            nodes.insert(reg, node);
        }

        // adding edge between physical registers
        let keys: Vec<_> = nodes.keys().cloned().collect();
        for i in 0..keys.len() {
            for j in (i + 1)..keys.len() {
                let reg1 = keys[i];
                let reg2 = keys[j];
                nodes.get_mut(&reg1).unwrap().neighbors.insert(reg2);
                nodes.get_mut(&reg2).unwrap().neighbors.insert(reg1);
            }
        }

        Self {
            nodes,
        }
    }
}

impl Rig {
    // 2. Add virtual registers into the graph.
    fn add_virtreg(&mut self, function: &Function) {
        for insn in function.body.iter() {
            match insn {
                Insn::Add(dst, src1, src2) |
                Insn::Addw(dst, src1, src2) |
                Insn::Sub(dst, src1, src2) |
                Insn::Subw(dst, src1, src2) |
                Insn::Mul(dst, src1, src2) |
                Insn::Mulw(dst, src1, src2) |
                Insn::Div(dst, src1, src2) |
                Insn::Divw(dst, src1, src2) |
                Insn::Rem(dst, src1, src2) |
                Insn::Remw(dst, src1, src2) |
                Insn::Slt(dst, src1, src2) |
                Insn::Sgt(dst, src1, src2) => {
                    self.proc_operand(dst);
                    self.proc_operand(src1);
                    self.proc_operand(src2);
                },
                Insn::Beq(src1, src2, ..) |
                Insn::Bne(src1, src2, ..) => {
                    self.proc_operand(src1);
                    self.proc_operand(src2);
                },
                Insn::Mv(dst, src) |
                Insn::Neg(dst, src) |
                Insn::Not(dst, src) |
                Insn::Negw(dst, src) |
                Insn::Sextw(dst, src) |
                Insn::Seqz(dst, src) |
                Insn::Snez(dst, src) => {
                    self.proc_operand(dst);
                    self.proc_operand(src);
                },
                Insn::Call(..) |
                Insn::Intermediate(..) |
                Insn::Ret |
                Insn::J(..) |
                Insn::Label(..) => {
                    ;
                },
                Insn::La(reg, name) |
                Insn::LoadStatic(reg, name) |
                Insn::StoreStatic(reg, name) => {
                    self.proc_operand(reg);
                },
                Insn::Ld(reg, mem) |
                Insn::Lw(reg, mem) |
                Insn::Sd(reg, mem) |
                Insn::Sw(reg, mem) => {
                    self.proc_operand(reg);
                    assert!(matches!(mem, Operand::Mem {..}));
                },
                Insn::Li(..) |
                Insn::Addi(..) |
                Insn::Addiw(..) => unreachable!(),
            }
        }
    }

    fn proc_operand(&mut self, operand: &Operand) {
        match operand {
            Operand::VirtReg(v_reg_id) => {
                let reg = GeneralReg::Virt(*v_reg_id);
                if !self.nodes.contains_key(&reg) {
                    let node = RigNode::new(reg, 0);
                    self.nodes.insert(reg, node);
                }
            },
            _ => {
                // nothing to do.
                ;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::lir::LirCodeGen;

    use super::*;

    #[test]
    fn test_rig_base() {
        let rig = Rig::base();
        for (reg, node) in rig.nodes.iter() {
            println!("{:?}: {:?}", reg, node.neighbors);
        }
    }

    fn gen_lir(path: &str) -> TopLevel {
        let input = std::fs::read_to_string(path).unwrap();
        let mut lexer = crate::lex::Lexer::new(input);
        let (tokens, strtb) = lexer.lex().unwrap();

        let mut parser = crate::ast::AstParser::new(tokens, strtb);
        let ast = parser.parse_prog().unwrap();

        let mut parser = crate::sem::HirParser::new();
        let hir = parser.parse(ast).unwrap();

        let mut parser = crate::tac::TacCodeGen::new();
        let (tac, _opt) = parser.parse(hir);

        let mut codegen_parse = LirCodeGen::new();
        let (lir, codegen_regalloc) = codegen_parse.parse(tac);

        lir
    }

    #[test]
    fn test_rig_add_virtreg() {
        let path = "../testprogs/reg.c";
        let lir = gen_lir(path);
        println!("{}", lir.emit_code());

        for (name, func) in lir.functions.iter() {
            let mut rig = Rig::base();
            rig.add_virtreg(func);
            println!("{:?}", rig.nodes.keys());
        }
    }
}