use std::collections::{BTreeMap, BTreeSet, HashMap};

use crate::common::*;
use super::{
    Function,
    Insn,
    Operand,
    LabelOperand,
};

#[derive(Debug)]
pub enum Node {
    Entry {
        successors: BTreeSet<NodeId>,
    },
    BasicBlock {
        id: usize,
        insns: Vec<Insn>,
        predecessors: BTreeSet<NodeId>,
        successors: BTreeSet<NodeId>,
    },
    Exit {
        predecessors: BTreeSet<NodeId>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeId {
    Entry,
    BasicBlock(usize),
    Exit,
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
        self.partial_cmp(other).expect("NodeId comparison failed")
    }
}

impl Node {
    pub fn id(&self) -> NodeId {
        match self {
            Node::Entry { .. } => NodeId::Entry,
            Node::BasicBlock { id, .. } => NodeId::BasicBlock(*id),
            Node::Exit { .. } => NodeId::Exit,
        }
    }

    pub fn basic_block(id: usize, insns: Vec<Insn>) -> Self {
        Node::BasicBlock {
            id,
            insns,
            predecessors: BTreeSet::new(),
            successors: BTreeSet::new(),
        }
    }
}

#[derive(Debug)]
pub struct Graph {
    pub nodes: BTreeMap<NodeId, Node>,
    pub label_map: HashMap<LabelOperand, NodeId>,
}

impl Graph {
    pub fn build(insns: Vec<Insn>) -> Self {
        let partition = Self::partition(insns);
        let edged = partition.add_edges();

        edged
    }

    fn partition(insns: Vec<Insn>) -> Self {
        let mut nodes = BTreeMap::new();
        let mut current_block = vec![];
        let mut current_id = 0;
        nodes.insert(NodeId::Entry, Node::Entry {
            successors: BTreeSet::new(),
        });
        let mut label_map = HashMap::new();

        for insn in insns {
            match insn {
                Insn::Label(label) => {
                    if !current_block.is_empty() {
                        nodes.insert(NodeId::BasicBlock(current_id), Node::basic_block(current_id, current_block));
                        current_id += 1;
                    }
                    current_block = vec![insn];
                    label_map.insert(label, NodeId::BasicBlock(current_id));
                },
                Insn::Jump(..)|Insn::Return(..)|
                Insn::BranchIfZero{..}|Insn::BranchNotZero{..} => {
                    current_block.push(insn);
                    nodes.insert(NodeId::BasicBlock(current_id), Node::basic_block(current_id, current_block));
                    current_id += 1;
                    current_block = vec![];
                }
                _ => current_block.push(insn),
            }
        }
    
        if !current_block.is_empty() {
            // panic!("Internal error: Basic block must end with a jump or return instruction");
            nodes.insert(NodeId::BasicBlock(current_id), Node::basic_block(current_id, current_block));
            current_id += 1;
        }

        nodes.insert(NodeId::Exit, Node::Exit {
            predecessors: BTreeSet::new(),
        });

        Self {
            nodes,
            label_map,
        }
    }

    fn add_edges(mut self) -> Self {
        // since we always insert a return statement at the end of a function,
        // so we can assume there is always a basic block 0.
        let mut nodes = self.nodes;
        let label_map = self.label_map;
        let last_basic_block_id = nodes.len() - 3;
        add_edge(&mut nodes, NodeId::Entry, NodeId::BasicBlock(0));

        let mut to_add = vec![];

        for (&id, node) in nodes.iter_mut() {
            match node {
                Node::Entry {..} | Node::Exit {..} => continue,
                Node::BasicBlock { 
                    id,
                    predecessors,
                    successors,
                    insns,
                } => {
                    let last_insn = insns.last()
                        .expect("Internal error: Basic block must have at least one instruction");
                    let next_id = if *id == last_basic_block_id {
                        NodeId::Exit
                    } else {
                        NodeId::BasicBlock(*id + 1)
                    };
                    match last_insn {
                        Insn::Return(..) => to_add.push((NodeId::BasicBlock(*id), NodeId::Exit)),
                        Insn::Jump(label) => {
                            let to_id = label_map.get(label)
                                .expect("Internal error: Jump label not found in label map");
                            to_add.push((NodeId::BasicBlock(*id), *to_id));
                        },
                        Insn::BranchIfZero { label, .. } |
                        Insn::BranchNotZero { label, .. } => {
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
            add_edge(&mut nodes, from, to);
        }

        Self {
            nodes,
            label_map,
        }
    }


}
fn add_edge(
    nodes: &mut BTreeMap<NodeId, Node>, 
    from: NodeId, 
    to: NodeId
) {
    {
        let from = nodes.get_mut(&from)
            .expect("Internal error: Node not found in graph");

        match from {
            Node::BasicBlock { successors, .. } => {
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
            Node::BasicBlock { predecessors, .. } => {
                predecessors.insert(from);
            },
            Node::Exit { predecessors, .. } => {
                predecessors.insert(from);
            },
            _ => panic!("Internal error: Entry node cannot have predecessors"),
        }
    }
}
impl Graph {
    pub fn emit(mut self) -> Vec<Insn> {
        let mut insns = vec![];
        
        for (_id, block) in self.nodes {
            match block {
                Node::Entry {..} | Node::Exit {..} => continue,
                Node::BasicBlock { insns: block_insns, .. } => {
                    insns.extend(block_insns);
                },
            }
        }

        insns
    }
}