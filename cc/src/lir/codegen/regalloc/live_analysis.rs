use std::{cmp::min, collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque}};

use crate::{asm::Register, common::*, lir::{codegen::regalloc::GeneralReg, lir::LabelOperand, IntermediateInsn}};
use super::{
    CodeGen,
    RegAlloc,
    Canonic,
    Spill,
    TopLevel,
    Function,
    Insn,
    FuncContext,
    Operand,
};

#[derive(Debug, Clone)]
pub struct LiveReg {
    inner: HashSet<GeneralReg>,
}


#[derive(Debug, Clone)]
pub struct Iter<'a> {
    inner: std::collections::hash_set::Iter<'a, GeneralReg>,
}

impl Iterator for Iter<'_> {
    type Item = GeneralReg;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().cloned()
    }
}

impl LiveReg {
    pub fn iter(&self) -> Iter {
        Iter {
            inner: self.inner.iter(),
        }
    }

    pub fn new() -> Self {
        LiveReg {
            inner: HashSet::new(),
        }
    }

    pub fn add(&mut self, reg: GeneralReg) {
        self.inner.insert(reg);
    }

    pub fn remove(&mut self, reg: GeneralReg) {
        self.inner.remove(&reg);
    }

    pub fn contains(&self, reg: GeneralReg) -> bool {
        self.inner.contains(&reg)
    }

    pub fn union_with(&mut self, other: &LiveReg) {
        self.inner.extend(other.inner.iter().cloned());
    }

    pub fn diff_with(&self, other: &Self) -> bool {
        if self.inner.len() != other.inner.len() {
            return true;
        }
        for reg in self.inner.iter() {
            if !other.contains(*reg) {
                return true;
            }
        }
        false
    }
}


#[derive(Debug)]
pub struct AnalyzeResult {
    pub block_infos: HashMap<usize, LiveReg>,
    pub insn_infos: HashMap<InsnId, LiveReg>,
}

#[derive(Debug)]
pub enum Node<'a> {
    Entry {
        successors: BTreeSet<NodeId>,
    },
    BasicBlock(BasicBlock<'a>),
    Exit {
        predecessors: BTreeSet<NodeId>,
    },
}

#[derive(Debug)]
pub struct BasicBlock<'a> {
    pub id: usize,
    pub insns: Vec<&'a Insn>,
    pub predecessors: BTreeSet<NodeId>,
    pub successors: BTreeSet<NodeId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeId {
    Entry,
    BasicBlock(usize),
    Exit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InsnId {
    pub block_id: usize,
    pub inblock_idx: usize,
}

impl InsnId {
    pub fn new(block_id: usize, inblock_idx: usize) -> Self {
        InsnId { block_id, inblock_idx }
    }
}

impl PartialOrd for NodeId {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        use std::cmp::Ordering;
        match (self, other) {
            (NodeId::Entry, NodeId::BasicBlock(..)) => Some(Ordering::Less),
            (NodeId::BasicBlock(..), NodeId::Entry) => Some(Ordering::Greater),
            (NodeId::Entry, NodeId::Exit) => Some(Ordering::Less),
            (NodeId::Exit, NodeId::Entry) => Some(Ordering::Greater),
            (NodeId::BasicBlock(a), NodeId::BasicBlock(b)) => a.partial_cmp(b),
            (NodeId::Exit, NodeId::BasicBlock(..)) => Some(Ordering::Greater),
            (NodeId::BasicBlock(..), NodeId::Exit) => Some(Ordering::Less),
            (NodeId::Entry, NodeId::Entry) => Some(Ordering::Equal),
            (NodeId::Exit, NodeId::Exit) => Some(Ordering::Equal),
        }
    }
}

impl Ord for NodeId {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl<'a> Node<'a> {
    pub fn id(&self) -> NodeId {
        match self {
            Node::Entry { .. } => NodeId::Entry,
            Node::BasicBlock(bb) => NodeId::BasicBlock(bb.id),
            Node::Exit { .. } => NodeId::Exit,
        }
    }

    pub fn basic_block(id: usize, insns: Vec<&'a Insn>) -> Self {
        Node::BasicBlock(BasicBlock {
            id,
            insns,
            predecessors: BTreeSet::new(),
            successors: BTreeSet::new(),
        })
    }
}

#[derive(Debug)]
pub struct Graph<'a> {
    pub nodes: BTreeMap<NodeId, Node<'a>>,
}

impl<'a> Graph<'a> {
    pub fn build(insns: &'a Vec<Insn>) -> Self {
        let (mut partition, label_map) = Self::partition(insns);
        let edged = partition.add_edges(label_map);

        edged
    }

    fn partition(
        insns: &'a Vec<Insn>
    ) -> (
        Self, 
        HashMap<LabelOperand, NodeId>
    ) {
        let mut nodes = BTreeMap::new();
        let mut cur_block = vec![];
        let mut cur_block_id = 0;
        nodes.insert(NodeId::Entry, Node::Entry {
            successors: BTreeSet::new(),
        });
        let mut label_map = HashMap::new();
    
        for insn in insns {
            match insn {
                Insn::Label(label) => {
                    if !cur_block.is_empty() {
                        nodes.insert(
                            NodeId::BasicBlock(cur_block_id), 
                            Node::basic_block(cur_block_id, cur_block),
                        );
                        cur_block_id += 1;
                    }
                    cur_block = vec![insn];
                    label_map.insert(*label, NodeId::BasicBlock(cur_block_id));
                },
                Insn::J(..)|Insn::Intermediate(IntermediateInsn::Epilogue)|
                Insn::Beq(..)|Insn::Bne(..) => {
                    cur_block.push(insn);
                    nodes.insert(
                        NodeId::BasicBlock(cur_block_id), 
                        Node::basic_block(cur_block_id, cur_block),
                    );
                    cur_block_id += 1;
                    cur_block = vec![];
                },
                Insn::Ret => unreachable!(),
                _ => cur_block.push(insn),
            }
        }

        if !cur_block.is_empty() {
            nodes.insert(
                NodeId::BasicBlock(cur_block_id), 
                Node::basic_block(cur_block_id, cur_block),
            );
            cur_block_id += 1;
        }

        nodes.insert(NodeId::Exit, Node::Exit {
            predecessors: BTreeSet::new(),
        });

        (Self {
            nodes,
        }, label_map)
    }

    fn add_edges(
        mut self,
        label_map: HashMap<LabelOperand, NodeId>,
    ) -> Self {
        let mut nodes = self.nodes;
        let last_basic_block_id = nodes.len() - 3;
        Self::add_edge(&mut nodes, NodeId::Entry, NodeId::BasicBlock(0));

        let mut to_add = vec![];

        for (&id, node) in nodes.iter_mut() {
            match node {
                Node::Entry {..} | Node::Exit {..} => continue,
                Node::BasicBlock(BasicBlock { 
                    id,
                    predecessors,
                    successors,
                    insns,
                }) => {
                    let last_insn = insns.last()
                        .expect("Internal error: Basic block must have at least one instruction");
                    let next_id = if *id == last_basic_block_id {
                        NodeId::Exit
                    } else {
                        NodeId::BasicBlock(*id + 1)
                    };
                    match last_insn {
                        Insn::Intermediate(IntermediateInsn::Epilogue) =>
                            to_add.push((NodeId::BasicBlock(*id), NodeId::Exit)),
                        Insn::J(label) => {
                            let to_id = label_map.get(label)
                                .expect("Internal error: Jump label not found in label map");
                            to_add.push((NodeId::BasicBlock(*id), *to_id));
                        },
                        Insn::Beq(.., label) |
                        Insn::Bne(.., label) => {
                            let branch_id = label_map.get(label)
                                .expect("Internal error: Branch label not found in label map");
                            to_add.push((NodeId::BasicBlock(*id), *branch_id));
                            to_add.push((NodeId::BasicBlock(*id), next_id));
                        }
                        _ => to_add.push((NodeId::BasicBlock(*id), next_id)),
                    }
                },
            }
        }

        for (from, to) in to_add {
            Self::add_edge(&mut nodes, from, to);
        }

        Self {
            nodes,
        }
    }

    fn add_edge(
        nodes: &mut BTreeMap<NodeId, Node<'a>>,
        from: NodeId,
        to: NodeId,
    ) {
        {
            let from = nodes.get_mut(&from)
                .expect("Internal error: Node not found in graph");

            match from {
                Node::BasicBlock(BasicBlock { successors, .. }) => {
                    successors.insert(to);
                },
                Node::Entry { successors, .. } => {
                    successors.insert(to);
                },
                _ => panic!("Internal error: Exit node cannot have successors"),
            }
        }

        {
            let to = nodes.get_mut(&to)
                .expect("Internal error: Node not found in graph");
            match to {
                Node::BasicBlock(BasicBlock { predecessors, .. }) => {
                    predecessors.insert(from);
                },
                Node::Exit { predecessors, .. } => {
                    predecessors.insert(from);
                },
                _ => panic!("Internal error: Entry node cannot have predecessors"),
            }
        }
    }
}

#[derive(Debug)]
pub struct LiveAnalysis<'a, 'b> {
    cfg: &'a Graph<'b>,
    func_cxs: &'a HashMap<StrDescriptor, FuncContext>,
    block_infos: HashMap<usize, LiveReg>,
    insn_infos: HashMap<InsnId, LiveReg>,
}

impl<'a, 'b> LiveAnalysis<'a, 'b> {
    pub fn new(
        cfg: &'a Graph<'b>,
        func_cxs: &'a HashMap<StrDescriptor, FuncContext>,
    ) -> Self {
        LiveAnalysis {
            cfg,
            func_cxs,
            block_infos: HashMap::new(),
            insn_infos: HashMap::new(),
        }
    }

    pub fn analyze(self) -> AnalyzeResult {
        let mut analyze = self;

        // initialize
        for (id, node) in analyze.cfg.nodes.iter() {
            match node {
                Node::Entry {..} | Node::Exit {..} => continue,
                Node::BasicBlock(block) => 
                    analyze.annotate_block(block.id, LiveReg::new()),
            }
        }

        analyze.iterate();

        AnalyzeResult {
            block_infos: analyze.block_infos,
            insn_infos: analyze.insn_infos,
        }
    }

    fn transfer(
        &mut self,
        initial: &LiveReg,
        block: &BasicBlock,
    ) {
        let mut current = initial.clone();

        for (inblock_id, insn) in block.insns.iter().enumerate().rev() {
            self.annotate_insn(
                InsnId::new(block.id, inblock_id), 
                current.clone(),
            );

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
                    (*dst).try_into().map(|reg| current.remove(reg));
                    (*src1).try_into().map(|reg| current.add(reg));
                    (*src2).try_into().map(|reg| current.add(reg));
                },
                Insn::Mv(dst, src) |
                Insn::Neg(dst, src) |
                Insn::Not(dst, src) |
                Insn::Negw(dst, src) |
                Insn::Sextw(dst, src) |
                Insn::Seqz(dst, src) |
                Insn::Snez(dst, src) => {
                    (*dst).try_into().map(|reg| current.remove(reg));
                    (*src).try_into().map(|reg| current.add(reg));
                },
                Insn::Beq(src1, src2, ..) |
                Insn::Bne(src1, src2, ..) => {
                    (*src1).try_into().map(|reg| current.add(reg));
                    (*src2).try_into().map(|reg| current.add(reg));
                },
                Insn::Ret |
                Insn::Addi(..) |
                Insn::Addiw(..) |
                Insn::La(..) |
                Insn::Li(..) => unreachable!(),
                Insn::Ld(reg, mem) |
                Insn::Lw(reg, mem) => {
                    (*reg).try_into().map(|reg| current.remove(reg));
                    assert!(matches!(mem, Operand::Mem {..}));
                },
                Insn::Sd(reg, mem) |
                Insn::Sw(reg, mem) => {
                    (*reg).try_into().map(|reg| current.add(reg));
                    assert!(matches!(mem, Operand::Mem {..}));
                },
                Insn::LoadStatic(reg, name) => {
                    (*reg).try_into().map(|reg| current.remove(reg));
                },
                Insn::StoreStatic(reg, name) => {
                    (*reg).try_into().map(|reg| current.add(reg));
                },
                Insn::Intermediate(..) |
                Insn::J(..) |
                Insn::Label(..) => {
                    ;
                }
                Insn::Call(target) => {
                    // we take a conservative approach - all caller-saved registers
                    // are considered been used
                    for caller_saved in Register::iter().filter(|r| r.is_caller_saved()) {
                        current.remove(GeneralReg::Phys(caller_saved));                    
                    }

                    let func_cx = self.func_cxs.get(target)
                        .expect("Internal error: Function context not found");

                    let arg_len = func_cx.type_.param_types.len().min(8);
                    for i in 0..arg_len {
                        current.add(GeneralReg::Phys(Register::a(i)));
                    }
                }
            }
        }

        self.annotate_block(block.id, current);
    }

    fn meet(
        &mut self,
        block: &BasicBlock,
    ) -> LiveReg {
        let mut initial = LiveReg::new();

        for &succ_id in block.successors.iter() {
            match succ_id {
                NodeId::Entry => panic!("Internal error: Entry node cannot be a successor"),
                NodeId::Exit => initial.add(GeneralReg::Phys(Register::A0)), // return value register
                NodeId::BasicBlock(succ_id) => {
                    let succ_live = self.retrieve_block_liveregs(succ_id)
                        .expect("Internal error: Block live registers not found");
                    initial.union_with(succ_live);
                },
            }
        }

        initial
    }

    fn iterate(&mut self) {
        let mut to_process = VecDeque::new();

        for (&_id, node) in self.cfg.nodes.iter().rev() {
            match node {
                Node::Entry {..} | Node::Exit {..} => continue,
                Node::BasicBlock(block) => {
                    to_process.push_back(block);
                    while let Some(b) = to_process.pop_front() {
                        let prev = self.retrieve_block_liveregs(b.id)
                            .expect("Internal error: BlockId not found in block_info")
                            .clone();
                        let initial = self.meet(b);
                        self.transfer(&initial, b);
                        let cur = self.retrieve_block_liveregs(b.id)
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

    fn annotate_block(
        &mut self,
        block_id: usize,
        live_regs: LiveReg,
    ) {
        self.block_infos.insert(block_id, live_regs);
    }

    fn annotate_insn(
        &mut self,
        insn_id: InsnId,
        live_regs: LiveReg,
    ) {
        self.insn_infos.insert(insn_id, live_regs);
    }

    fn retrieve_block_liveregs(
        &self,
        block_id: usize,
    ) -> Option<&LiveReg> {
        self.block_infos.get(&block_id)
    }
}

pub fn analysis(
    cfg: &Graph,
    func_cxs: &HashMap<StrDescriptor, FuncContext>,
) -> AnalyzeResult {
    let live_analysis = LiveAnalysis::new(cfg, func_cxs);
    live_analysis.analyze()
}