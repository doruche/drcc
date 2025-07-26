use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use crate::{common::*, lir::{codegen::regalloc::GeneralReg, lir::LabelOperand, IntermediateInsn}};
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

#[derive(Debug)]
pub struct LiveReg {
    inner: HashSet<GeneralReg>,
}

#[derive(Debug)]
pub struct AnalyzeResult {

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