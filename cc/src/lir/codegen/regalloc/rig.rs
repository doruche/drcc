use std::collections::HashMap;

use crate::{asm::Register, common::*, lir::codegen::regalloc::live_analysis::{self, Graph, InsnId, LiveAnalysis, LiveReg}};
use super::{
    Rig,
    RigNode,
    GeneralReg,
    TopLevel,
    Function,
    Insn,
    Operand,
    FuncContext,
    AnalyzeResult,
};

// We'll split the procedure into several parts:
// 1. Build a base interference graph with all physical registers.
// 2. Add virtual registers into the graph.
// 3. Liveness analysis, with a control flow graph.
// 4. Add edges between virtual registers.
impl<'a> Rig<'a> {
    pub fn analyze(
        self,
        func: &Function, 
    ) -> AnalyzeResult {
        let mut rig = self;
        rig.add_virtreg(func);

        let cfg = Graph::build(&func.body);
        let live_analysis = LiveAnalysis::new(&cfg, rig.func_cxs);
        let live_result = live_analysis.analyze();
        rig.add_edges(&cfg, live_result.insn_infos);
        rig.calc_spill_cost();

        let rig = rig.color();

        let map = rig.nodes.into_iter()
            .map(|(reg, node)| 
                (reg, node.color)
            )
            .collect();

        AnalyzeResult {
            map,
        }
    }

    pub fn base(func_cxs: &'a HashMap<StrDescriptor, FuncContext>) -> Self {
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
            func_cxs,
        }
    }

    fn contains(&self, reg: GeneralReg) -> bool {
        self.nodes.contains_key(&reg)
    }

    fn add_edge(&mut self, reg1: GeneralReg, reg2: GeneralReg) {
        if reg1 == reg2 {
            return;
        }
        if let Some(node1) = self.nodes.get_mut(&reg1) {
            node1.neighbors.insert(reg2);
        } else { panic!("Internal error: {:?} not found", reg1) }
        if let Some(node2) = self.nodes.get_mut(&reg2) {
            node2.neighbors.insert(reg1);
        } else { panic!("Internal error: {:?} not found", reg2) }
    }
}

impl<'a> Rig<'a> {
    // 3. Add edges between virtual registers.
    fn add_edges(
        &mut self, 
        cfg: &Graph,
        insn_infos: HashMap<InsnId, LiveReg>,
    ) {
        use live_analysis::Node as CfgNode;

        for (node_id, node) in cfg.nodes.iter() {
            match node {
                CfgNode::Entry {..} | CfgNode::Exit {..} => continue,
                CfgNode::BasicBlock(block) => {
                    for (inblock_id, &insn) in block.insns.iter().enumerate() {
                        let live_regs = insn_infos.get(&InsnId::new(block.id, inblock_id))
                            .expect("Internal error: Insn not found in live analysis");
                        match insn {
                            Insn::Add(dst, ..) |
                            Insn::Addw(dst, ..) |
                            Insn::Sub(dst, ..) |
                            Insn::Subw(dst, ..) |
                            Insn::Mul(dst, ..) |
                            Insn::Mulw(dst, ..) |
                            Insn::Div(dst, ..) |
                            Insn::Divw(dst, ..) |
                            Insn::Rem(dst, ..) |
                            Insn::Remw(dst, ..) |
                            Insn::Slt(dst, ..) |
                            Insn::Sgt(dst, ..) |
                            Insn::LoadStatic(dst, ..) |
                            Insn::Ld(dst, ..) |
                            Insn::Lw(dst, ..) |
                            Insn::Neg(dst, ..) |
                            Insn::Negw(dst, ..) |
                            Insn::Mv(dst, ..) |
                            Insn::Not(dst, ..) |
                            Insn::Sextw(dst, ..) |
                            Insn::Seqz(dst, ..) |
                            Insn::Snez(dst, ..) => {
                                for live_reg in live_regs.iter() {
                                    (*dst).try_into().map(|updated_reg| {
                                        self.add_edge(updated_reg, live_reg);
                                    });
                                }
                            },
                            Insn::Call(target) => {
                                let func_cx = self.func_cxs.get(target)
                                    .expect("Internal error: Function context not found");

                                for caller_saved in Register::iter().filter(|r| r.is_caller_saved()) {
                                    let caller_saved_reg = GeneralReg::Phys(caller_saved);
                                    for live_reg in live_regs.iter() {
                                        self.add_edge(caller_saved_reg, live_reg);
                                    }
                                }
                            },
                            Insn::Addi(..) |
                            Insn::Addiw(..) |
                            Insn::La(..) |
                            Insn::Li(..) |
                            Insn::Ret => unreachable!(),
                            Insn::Beq(..) |
                            Insn::Bne(..) |
                            Insn::J(..) |
                            Insn::Label(..) |
                            Insn::Intermediate(..) |
                            Insn::Sd(..) |
                            Insn::Sw(..) |
                            Insn::StoreStatic(..) => {
                                ;
                            },
                        }
                    }
                }                
            }
        }
    }

    fn calc_spill_cost(&mut self) {
        // we use a simple heuristic here:
        // the spill cost is the number of neighbors in the interference graph.
        // for hard registers, we set the spill cost to an infinite value.
        for node in self.nodes.values_mut() {
            if let GeneralReg::Phys(_) = node.reg {
                node.spill_cost = usize::MAX;
            } else {
                node.spill_cost = node.neighbors.len();
            }
        }
    }
}

impl<'a> Rig<'a> {
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

impl<'a> Rig<'a> {
    fn color(mut self) -> Self {
        let mut nodes: Vec<_> = self.nodes
            .into_iter()
            .map(|(_, node)| node)
            .collect();
        
        let colored = Self::color_inner(nodes);

        let nodes = colored.into_iter()
            .map(|node| {
                let reg = node.reg;
                (reg, node)
            })
            .collect();

        Self {
            nodes,
            func_cxs: self.func_cxs,
        }
    }

    fn color_inner(nodes: Vec<RigNode>) -> Vec<RigNode> {
        if nodes.len() == 0 {
            return nodes;
        }
        
        let (mut to_prune, unpruned) = Self::prune(nodes);

        let mut unpruned = Self::color_inner(unpruned);
    
        // color the pruned node
        if let Some(node) = &mut to_prune {
            let colors = Register::ALLOC_REGS.iter()
                .filter(|r| !node.neighbors.contains(&GeneralReg::Phys(***r)))
                .cloned()
                .collect::<Vec<_>>();

            if let Some(color) = colors.first() {
                node.color = Some(**color);
            }
        } else { unreachable!() }

        if let Some(node) = to_prune {
            unpruned.push(node);
        }

        unpruned
    }
    
    fn prune(
        nodes: Vec<RigNode>,
    ) -> (
        Option<RigNode>, // pruned
        Vec<RigNode>, // unpruned
    ) {
        if nodes.len() == 0 {
            return (None, vec![]);
        }

        let mut to_prune = None;
        let mut unpruned = vec![];

        for node in nodes {
            if let None = to_prune {
                if node.neighbors.len() < Register::ALLOC_REGS.len() {
                    to_prune = Some(node);
                } else {
                    unpruned.push(node);
                }
            } else {
                unpruned.push(node);
            }
        }

        if let None = to_prune {
            // spill
            let mut metric = usize::MAX;
            let mut to_prune_idx = 0;
            for (idx, node) in unpruned.iter().enumerate() {
                if node.spill_cost < metric {
                    metric = node.spill_cost;
                    to_prune_idx = idx;
                }
            }
            to_prune = Some(unpruned.remove(to_prune_idx));
        }

        (to_prune, unpruned)
    }

}

#[cfg(test)]
mod tests {
    use std::hash::Hash;

    use crate::lir::{codegen::{regalloc::live_analysis::Graph, RegAlloc}, LirCodeGen};

    use super::*;

    #[test]
    fn test_rig_base() {
        let func_cxs = HashMap::new();
        let rig = Rig::base(&func_cxs);
        for (reg, node) in rig.nodes.iter() {
            println!("{:?}: {:?}", reg, node.neighbors);
        }
    }

    fn gen_lir(path: &str) -> (TopLevel, LirCodeGen<RegAlloc>) {
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
        codegen_parse.parse(tac)
    }

    #[test]
    fn test_cfg() {
        let path = "../testprogs/control_flow.c";
        let (lir, _) = gen_lir(path);
        println!("{}", lir.emit_code());
        println!();
        for func in lir.functions.values() {
            let cfg = Graph::build(&func.body);
            println!("{:#?}", cfg.nodes);
        }   
    }

    #[test]
    fn test_rig() {
        let path = "../testprogs/control_flow.c";
        let (lir, codegen_regalloc) = gen_lir(path);
        let rig = Rig::base(&codegen_regalloc.func_cxs);
    }
}