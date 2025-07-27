use std::backtrace;
use std::collections::{HashMap, HashSet, VecDeque};

use crate::common::*;
use crate::tac::opt::cfg::InsnId;
use super::{
    Operand,
    Insn,
    Function,
    TopLevel,
    UnaryOp,
    BinaryOp,
    LabelOperand,
    Opt,
    CodeGen,
};
use super::cfg::{
    Node,
    NodeId,
    Graph,
    BasicBlock,
};

#[derive(Debug, Clone)]
struct LiveVars {
    inner: HashSet<GeneralVar>,
}

impl LiveVars {
    fn new() -> Self {
        LiveVars { inner: HashSet::new() }
    }

    fn diff_with(&self, other: &Self) -> bool {
        self.inner != other.inner
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum GeneralVar {
    Var {
        name: StrDescriptor,
        local_id: Option<usize>,
        data_type: DataType,
    },
    Temp(usize, DataType),
}

impl From<GeneralVar> for Operand {
    fn from(var: GeneralVar) -> Self {
        match var {
            GeneralVar::Var { name, local_id, data_type } => Operand::Var { name, local_id, data_type },
            GeneralVar::Temp(id, data_type) => Operand::Temp(id, data_type),
        }
    }
}

impl TryFrom<Operand> for GeneralVar {
    type Error = ();

    fn try_from(op: Operand) -> std::result::Result<GeneralVar, ()> {
        match op {
            Operand::Var { name, local_id, data_type } => Ok(GeneralVar::Var { name, local_id, data_type }),
            Operand::Temp(id, data_type) => Ok(GeneralVar::Temp(id, data_type)),
            _ => Err(()),
        }
    }
}


#[derive(Debug)]
struct LiveAnalysis<'a> {
    cfg: &'a Graph,
    block_info: HashMap<usize, LiveVars>,
    insn_info: HashMap<InsnId, LiveVars>,
    static_vars: LiveVars,
}

#[derive(Debug)]
struct AnalysisResult {
    block_info: HashMap<usize, LiveVars>,
    insn_info: HashMap<InsnId, LiveVars>,
}

impl<'a> LiveAnalysis<'a> {
    fn new(cfg: &'a Graph, static_vars: LiveVars) -> Self {
        LiveAnalysis {
            cfg,
            block_info: HashMap::new(),
            insn_info: HashMap::new(),
            static_vars,
        }
    }

    fn analyze(self) -> AnalysisResult {
        let mut analysis = self;

        // initialize the block_info with empty live sets
        for (&id, node) in analysis.cfg.nodes.iter() {
            match node {
                Node::Entry {..} | Node::Exit {..} => continue,
                Node::BasicBlock(block) => {
                    analysis.block_info.insert(block.id, LiveVars::new());
                }
            }
        }
    
        analysis.iterate();

        AnalysisResult {
            block_info: analysis.block_info,
            insn_info: analysis.insn_info,
        }
    }


    fn transfer(
        &mut self,
        initial: &LiveVars,
        block: &BasicBlock,
    ) {
        let mut current = initial.clone();

        for (inblock_id, insn) in block.insns.iter().enumerate().rev() {
            self.annotate_insn(
                InsnId::new(block.id, inblock_id), 
                current.clone()
            );

            match insn {
                Insn::Move { dst, src } |
                Insn::SignExt { dst, src } |
                Insn::Truncate { dst, src } |
                Insn::Unary { dst, src, ..} => {
                    (*dst).try_into().map(|var| {
                        current.inner.remove(&var);
                    });
                    (*src).try_into().map(|var| {
                        current.inner.insert(var);
                    });
                },
                Insn::Binary {
                    dst,
                    left,
                    right,
                    ..
                } => {
                    (*dst).try_into().map(|var| {
                        current.inner.remove(&var);
                    });
                    (*left).try_into().map(|var| {
                        current.inner.insert(var);
                    });
                    (*right).try_into().map(|var| {
                        current.inner.insert(var);
                    });
                },
                Insn::FuncCall { dst, args, .. } => {
                    // cz we don't know whether the function reads static variables,
                    // we take a conservative approach - add all static variables
                    // to the live set.
                    (*dst).try_into().map(|var| {
                        current.inner.remove(&var);
                    });
                    for &arg in args {
                        arg.try_into().map(|var| {
                            current.inner.insert(var);
                        });
                    }
                    
                    current.inner.extend(self.static_vars.inner.iter().cloned());
                }
                Insn::BranchIfZero { src, .. } |
                Insn::BranchNotZero { src, .. } |
                Insn::Return(src) => {
                    (*src).try_into().map(|var| {
                        current.inner.insert(var);
                    });
                    println!("Live variable before {:?}: {:?}", insn, current.inner);
                },
                _ => {
                    // other instructions do not affect live variables
                    ;
                }
            }
        }

        self.annotate_block(block.id, current);
    }

    fn meet(
        &mut self,
        block: &BasicBlock,
    ) -> LiveVars {
        let mut initial = HashSet::new();

        for succ_id in block.successors.iter() {
            match succ_id {
                NodeId::Entry => panic!("Internal error: Entry node should not be a successor"),
                NodeId::Exit => initial.extend(self.static_vars.inner.iter().cloned()),
                NodeId::BasicBlock(id) => {
                    if let Some(live_vars) = self.block_info.get(&id) {
                        initial.extend(live_vars.inner.iter().cloned());
                    } else { panic!("Internal error: BlockId not found in block_info") }
                }
            }
        }

        LiveVars { inner: initial }
    }

    fn iterate(&mut self) {
        let mut to_process = VecDeque::new();

        for (&_id, node) in self.cfg.nodes.iter().rev() {
            match node {
                Node::Entry {..} | Node::Exit {..} => continue,
                Node::BasicBlock(block) => {
                    to_process.push_back(block);
                    self.annotate_block(block.id, LiveVars::new());
                    while let Some(b) = to_process.pop_front() {
                        let prev = self.retrieve_block_livevars(b.id)
                            .expect("Internal error: BlockId not found in block_info")
                            .clone();
                        let initial = self.meet(b);
                        self.transfer(&initial, b);
                        let cur = self.retrieve_block_livevars(b.id)
                            .expect("Internal error: BlockId not found in block_info");

                        if prev.diff_with(cur) {
                            for pred_id in b.predecessors.iter() {
                                match pred_id {
                                    NodeId::Entry => continue,
                                    NodeId::Exit => panic!("Internal error: Exit node should not be a predecessor"),
                                    id@NodeId::BasicBlock(..) => {
                                        let pred_block = self.cfg.nodes.get(id)
                                            .expect("Internal error: BlockId not found in cfg");
                                        if let Node::BasicBlock(pred_block) = pred_block {
                                            if to_process.iter().all(|b| b.id != pred_block.id) {
                                                to_process.push_back(pred_block);
                                            }
                                        } else { unreachable!() }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

    }

    fn annotate_insn(
        &mut self,
        insn_id: InsnId,
        live_vars: LiveVars,
    ) {
        self.insn_info.insert(insn_id, live_vars);
    }

    fn annotate_block(
        &mut self,
        block_id: usize,
        live_vars: LiveVars,
    ) {
        self.block_info.insert(block_id, live_vars);
    }

    fn retrieve_block_livevars(
        &self,
        block_id: usize,
    ) -> Option<&LiveVars> {
        self.block_info.get(&block_id)
    }
}

impl CodeGen<Opt> {
    pub fn deadstore_elimination(&mut self, func: Function) -> Function {
        match func {
            Function::Declared {..} => return func,
            Function::Defined {
                return_type,
                linkage,
                name,
                params,
                local_vars,
                body,
            } => {
                let cfg = Graph::build(body);

                let static_vars = self.static_vars.iter()
                    .map(|&(name, data_type)| GeneralVar::Var {
                        name,
                        local_id: None,
                        data_type,
                    })
                    .collect::<HashSet<_>>();
                let static_vars = LiveVars { inner: static_vars };

                let analysis = LiveAnalysis::new(&cfg, static_vars);
                let AnalysisResult {
                    block_info,
                    insn_info,
                } = analysis.analyze();

                let opted_cfg = rewrite_graph(cfg, &block_info, &insn_info);

                let opted_body = opted_cfg.emit();

                Function::Defined {
                    name,
                    params,
                    return_type,
                    body: opted_body,
                    linkage,
                    local_vars,
                }
            },
        }
    }
}

fn rewrite_graph(
    cfg: Graph,
    block_infos: &HashMap<usize, LiveVars>,
    insn_infos: &HashMap<InsnId, LiveVars>,
) -> Graph {
    let mut cfg = cfg;

    cfg.nodes = cfg.nodes.into_iter()
        .map(|(id, node)| 
            (id, rewrite_node(node, block_infos, insn_infos)))
        .collect();

    cfg
}

fn rewrite_node(
    node: Node,
    block_infos: &HashMap<usize, LiveVars>,
    insn_infos: &HashMap<InsnId, LiveVars>,
) -> Node {
    match node {
        Node::Entry {..} | Node::Exit {..} => node,
        Node::BasicBlock(mut block) => {
            block.insns = rewrite_insns(
                block.id,
                block.insns,
                block_infos,
                insn_infos,
            );
            Node::BasicBlock(block)
        }
    }
}

fn rewrite_insns(
    block_id: usize,
    insns: Vec<Insn>,
    block_infos: &HashMap<usize, LiveVars>,
    insn_infos: &HashMap<InsnId, LiveVars>,
) -> Vec<Insn> {
    insns.into_iter()
        .enumerate()
        .filter_map(|(inblocl_id, insn)| {
            let insn_id = InsnId::new(block_id, inblocl_id);
            let insn_info = insn_infos.get(&insn_id)
                .expect("Internal error: InsnId not found in insn_infos");
            rewrite_insn(insn, insn_info)
        })
        .collect()
}

fn rewrite_insn(
    insn: Insn,
    insn_info: &LiveVars,
) -> Option<Insn> {
    match insn {
        f@Insn::FuncCall {..} => Some(f),
        Insn::Unary { dst, .. } |
        Insn::Binary { dst, .. } |
        Insn::Move { dst, ..} |
        Insn::SignExt { dst, .. } |
        Insn::Truncate { dst, .. } => {
            if let Ok(var) = dst.try_into() {
                if insn_info.inner.contains(&var) {
                    Some(insn)
                } else {
                    None
                }
            } else {
                Some(insn)
            }
        },
        _ => Some(insn),
    }
}