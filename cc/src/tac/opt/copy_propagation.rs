use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};

use crate::common::*;
use crate::tac::opt::cfg::{BasicBlock, InsnId};
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
};

#[derive(Debug, Clone)]
struct ReachingCopies {
    defs: HashSet<(Operand, Operand)>,
}

impl From<HashSet<(Operand, Operand)>> for ReachingCopies {
    fn from(defs: HashSet<(Operand, Operand)>) -> Self {
        ReachingCopies { defs }
    }
}

#[derive(Debug)]
struct Iter<'a> {
    reaching_copies: std::collections::hash_set::Iter<'a, (Operand, Operand)>,
}

impl<'a> Iterator for Iter<'a> {
    /// (dst, src)
    type Item = (Operand, Operand);

    fn next(&mut self) -> Option<Self::Item> {
        self.reaching_copies.next().cloned()
    }
}

impl ReachingCopies {
    fn new() -> Self {
        ReachingCopies {
            defs: HashSet::new(),
        }
    }

    fn add(&mut self, dst: Operand, src: Operand) {
        self.defs.insert((dst, src));
    }

    fn remove(&mut self, dst: &Operand, src: &Operand) {
        self.defs.remove(&(*dst, *src));
    }

    fn contains(&self, dst: Operand, src: Operand) -> bool {
        self.defs.contains(&(dst, src))
    }

    fn iter(&self) -> Iter {
        Iter {
            reaching_copies: self.defs.iter(),
        }
    }

    fn intersect_with(&mut self, other: &Self) {
        let mut to_remove = vec![];
        for (dst, src) in self.iter() {
            if !other.contains(dst, src) {
                to_remove.push((dst, src));
            }
        }
        for (dst, src) in to_remove {
            self.remove(&dst, &src);
        }
    }

    fn diff_with(&self, other: &Self) -> bool {
        if self.defs.len() != other.defs.len() {
            return true;
        }
        for (dst, src) in self.iter() {
            if !other.contains(dst, src) {
                return true;
            }
        }
        false
    }
}

struct CopyPropagation<'a> {
    cfg: &'a Graph,
    block_defs: HashMap<usize, ReachingCopies>,
    insn_defs: HashMap<InsnId, ReachingCopies>,

    // At the creation of the analysis, we set this field to all copies in the function.
    // When other blocks initialize, they will clone this map as their initial state.
    // This serves as a cache to avoid recomputing the initial copies for each block.
    initial_copies: ReachingCopies,
}

struct AnalysisResult {
    block_defs: HashMap<usize, ReachingCopies>,
    insn_defs: HashMap<InsnId, ReachingCopies>,
}

impl<'a> CopyPropagation<'a> {
    fn new(cfg: &'a Graph) -> Self {
        fn initial_copies(cfg: &Graph) -> ReachingCopies {
            cfg.nodes
                .values()
                .filter_map(|node| {
                    if let Node::BasicBlock(BasicBlock { insns, .. }) = node {
                        Some(
                            insns.iter()
                                .filter_map(|insn| {
                                    if let Insn::Move { dst, src } = insn {
                                        Some((*dst, *src))
                                    } else {
                                        None
                                    }
                                })
                                .collect::<HashSet<_>>()
                                .into(),
                        )
                    } else {
                        None
                    }
                })
                .fold(ReachingCopies::new(), |mut acc, defs: ReachingCopies| {
                    for (dst, src) in defs.defs {
                        acc.add(dst, src);
                    }
                    acc
                })   
        }
        let initial_copies = initial_copies(cfg);

        CopyPropagation {
            cfg,
            block_defs: HashMap::new(),
            insn_defs: HashMap::new(),
            initial_copies,
        }
    }

    fn analyze(self) -> AnalysisResult {
        let mut analysis = self;

        // initialize all blocks with the initial copies
        for (&id, node) in analysis.cfg.nodes.iter() {
            match node {
                Node::BasicBlock(block) => {
                    analysis.block_defs.insert(block.id, analysis.initial_copies.clone());
                },
                Node::Entry { .. } | Node::Exit { .. } => continue,
            }
        }

        analysis.iterate();

        AnalysisResult {
            block_defs: analysis.block_defs,
            insn_defs: analysis.insn_defs,
        }
    }

    fn transfer(
        &mut self,
        initial: &ReachingCopies,
        basic_block: &BasicBlock,
    ) {
        let mut current = initial.clone();
        
        for (inblock_id, insn) in basic_block.insns.iter().enumerate() {
            self.annotate_insn(
                InsnId::new(basic_block.id, inblock_id), 
                current.clone(),
            );

            // transfer
            match insn {
                Insn::Move { src, dst } => {
                    if current.contains(*dst, *src) {
                        // we have x = y already, so no need to propagate y = x.
                        continue;
                    }
                    
                    // kill conflicting copies
                    {
                        let mut to_remove = vec![];
                        for (d, s) in current.iter() {
                            if d == *dst || s == *dst {
                                to_remove.push((d, s));
                            }
                        }
                        for (d, s) in to_remove {
                            current.remove(&d, &s);
                        }
                    }

                    current.add(*dst, *src);
                },
                Insn::FuncCall { dst, .. } => {  
                    // not only do we need to remove conflicting copies,
                    // but we also need to remove those that are related to static variables,
                    // cz we don't know if the function will modify them, so we choose a 
                    // conservative approach.
                    let mut to_remove = vec![];
                    for (d, s) in current.iter() {
                        if *dst == d || *dst == s || d.is_static() || s.is_static() {
                            to_remove.push((d, s));
                        }
                    }
                    for (d, s) in to_remove {
                        current.remove(&d, &s);
                    }
                },
                Insn::SignExt { dst, .. } |
                Insn::Truncate { dst, .. } |
                Insn::Unary { dst, .. } |
                Insn::Binary { dst, ..} => {
                    let mut to_remove = vec![];
                    for (d, s) in current.iter() {
                        if *dst == d || *dst == s {
                            to_remove.push((d, s));
                        }
                    }
                    for (d, s) in to_remove {
                        current.remove(&d, &s);
                    }
                },
                _ => {
                    // other instructions do not involve assignments,
                    // so we do not need to modify the reaching copies.
                    ;
                }
            }
        }

        self.annotate_block(basic_block.id, current);
    }

    fn meet(
        &self,
        basic_block: &BasicBlock
    ) -> ReachingCopies {
        let mut initial = self.initial_copies.clone();
        
        for pred in basic_block.predecessors.iter() {
            match pred {
                NodeId::Entry => return ReachingCopies::new(),
                NodeId::Exit => panic!("Internal error: Exit node should not be a predecessor"),
                NodeId::BasicBlock(id) => {
                    let pred = self.block_defs.get(id)
                        .expect("Internal error: Basic block not found in block_defs");
                    initial.intersect_with(pred);
                }
            }
        }
        
        initial
    }

    fn iterate(&mut self) {
        let mut to_process = VecDeque::new();

        for (&_id, node) in self.cfg.nodes.iter() {
            match node {
                Node::Entry { .. } => continue,
                Node::Exit { .. } => continue,
                Node::BasicBlock(block) => {
                    to_process.push_back(block);
                    while let Some(b) = to_process.pop_front() {
                        let prev = self.retrieve_block_defs(b.id)
                            .expect("Internal error: Basic block not found in block_defs")
                            .clone();
                        
                        let incoming = self.meet(b);
                        self.transfer(&incoming, b);
                        
                        let cur = self.block_defs.get(&b.id)
                            .expect("Internal error: Basic block not found in block_defs");
                        if prev.diff_with(cur) {
                            for succ in b.successors.iter() {
                                match succ {
                                    NodeId::Entry => panic!("Internal error: Entry node should not be a successor"),
                                    NodeId::Exit => continue,
                                    succ_id@NodeId::BasicBlock(..) => {
                                        let succ_block = self.cfg.nodes.get(succ_id)
                                            .expect("Internal error: Basic block not found in cfg");
                                        if let Node::BasicBlock(succ_block) = succ_block {
                                            if to_process.iter().all(|b| b.id != succ_block.id) {
                                                to_process.push_back(succ_block);
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
        reaching_copies: ReachingCopies,
    ) {
        self.insn_defs.insert(insn_id, reaching_copies);
    }

    fn annotate_block(
        &mut self,
        block_id: usize,
        reaching_copies: ReachingCopies,
    ) {
        self.block_defs.insert(block_id, reaching_copies);
    }

    fn retrieve_insn_defs(
        &self,
        insn_id: InsnId,
    ) -> Option<&ReachingCopies> {
        self.insn_defs.get(&insn_id)
    }

    fn retrieve_block_defs(
        &self,
        block_id: usize,
    ) -> Option<&ReachingCopies> {
        self.block_defs.get(&block_id)
    }
}

impl CodeGen<Opt> {
    pub fn copy_propagation(&mut self, func: Function) -> Function {
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

                let analysis = CopyPropagation::new(&cfg);
                let result = analysis.analyze();

                let opted_cfg = rewrite_graph(cfg, &result.block_defs, &result.insn_defs);

                let mut opted_body = opted_cfg.emit();

                Function::Defined {
                    name,
                    params,
                    return_type,
                    body: opted_body,
                    linkage,
                    local_vars,
                }
            }
        }
    }
}

fn rewrite_graph(
    cfg: Graph,
    block_defs: &HashMap<usize, ReachingCopies>,
    insn_defs: &HashMap<InsnId, ReachingCopies>,
) -> Graph {
    let mut cfg = cfg;

    cfg.nodes = cfg.nodes.into_iter()
        .map(|(id, node)| {
            let rewritten_node = rewrite_node(node, block_defs, insn_defs);
            (id, rewritten_node)
        })
        .collect::<BTreeMap<_, _>>();
    
    cfg
}

fn rewrite_node(
    node: Node,
    block_defs: &HashMap<usize, ReachingCopies>,
    insn_defs: &HashMap<InsnId, ReachingCopies>,
) -> Node {
    match node {
        Node::Entry { .. } | Node::Exit { .. } => node,
        Node::BasicBlock(mut block) => {
            block.insns = rewrite_insns(
                block.id,
                block.insns,
                block_defs,
                insn_defs,
            );
            Node::BasicBlock(block)
        }
    }
}

fn rewrite_insns(
    block_id: usize,
    insns: Vec<Insn>,
    block_defs: &HashMap<usize, ReachingCopies>,
    insn_defs: &HashMap<InsnId, ReachingCopies>,
) -> Vec<Insn> {
    insns.into_iter()
        .enumerate()
        .filter_map(|(inblock_id, insn)| {
            let insn_id = InsnId::new(block_id, inblock_id);
            let reaching_copies = insn_defs.get(&insn_id)
                .expect("Internal error: Reaching copies not found for instruction");

            rewrite_insn(insn, reaching_copies)
        })
        .collect()
}

fn rewrite_insn(
    insn: Insn,
    reaching_copies: &ReachingCopies,
) -> Option<Insn> {
    match insn {
        Insn::Move { src, dst } => {
            if reaching_copies.contains(dst, src) |
                reaching_copies.contains(src, dst)  {
                None
            } else {
                Some(Insn::Move {
                    src: rewrite_operand(src, reaching_copies),
                    dst,
                })
            }
        },
        Insn::Unary { 
            op, 
            dst, 
            src 
        } => Some(Insn::Unary {
            op,
            dst,
            src: rewrite_operand(src, reaching_copies),
        }),
        Insn::Binary { 
            op, 
            left, 
            right, 
            dst 
        } => Some(Insn::Binary {
            op,
            left: rewrite_operand(left, reaching_copies),
            right: rewrite_operand(right, reaching_copies),
            dst,
        }),
        Insn::SignExt { 
            dst, 
            src 
        } => Some(Insn::SignExt {
            dst,
            src: rewrite_operand(src, reaching_copies),
        }),
        Insn::Truncate { 
            dst, 
            src 
        } => Some(Insn::Truncate {
            dst,
            src: rewrite_operand(src, reaching_copies),
        }),
        Insn::FuncCall { 
            dst, 
            args, 
            target,
        } => {
            let rewritten_args = args.into_iter()
                .map(|arg| rewrite_operand(arg, reaching_copies))
                .collect();
            Some(Insn::FuncCall {
                dst,
                args: rewritten_args,
                target,
            })
        },
        Insn::Return(ret_val) => Some(Insn::Return(rewrite_operand(ret_val, reaching_copies))),
        Insn::BranchIfZero { src, label } => Some(Insn::BranchIfZero {
            src: rewrite_operand(src, reaching_copies),
            label,
        }),
        Insn::BranchNotZero { src, label } => Some(Insn::BranchNotZero {
            src: rewrite_operand(src, reaching_copies),
            label,
        }),
        _ => Some(insn),
    }
}

fn rewrite_operand(
    operand: Operand,
    reaching_copies: &ReachingCopies,
) -> Operand {
    match operand {
        Operand::Imm(..) => operand,
        _ => {
            if let Some((_, src)) = reaching_copies.iter().find(|(dst, _)| *dst == operand) {
                src
            } else {
                operand
            }
        }
    }
}