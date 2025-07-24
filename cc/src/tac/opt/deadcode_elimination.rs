use std::collections::{BTreeSet, VecDeque};

use crate::common::*;
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

impl CodeGen<Opt> {
    pub fn deadcode_elimination(&mut self, func: Function) -> Function {
        let mut cfg = Graph::build(func.body);

        cfg = prune_blocks(cfg);
        cfg = remove_useless_jumps(cfg);
        cfg = remove_useless_labels(cfg);

        let mut opted_body = cfg.emit();

        Function {
            name: func.name,
            params: func.params,
            return_type: func.return_type,
            body: opted_body,
            linkage: func.linkage,
            local_vars: func.local_vars,
        }
    }
}

fn prune_blocks(cfg: Graph) -> Graph {
    // simple bfs

    let mut nodes = cfg.nodes;
    let mut visited = BTreeSet::new();
    let mut to_visit_id = VecDeque::new();
    to_visit_id.push_back(NodeId::Entry);

    while let Some(id) = to_visit_id.pop_front() {
        if visited.contains(&id) {
            continue;
        }
        visited.insert(id);

        let node = nodes.get(&id).expect("Internal error: Node not found in graph");
        match node {
            Node::Entry { successors } |
            Node::BasicBlock { successors, .. } => {
                for succ in successors.iter() {
                    if !visited.contains(succ) {
                        to_visit_id.push_back(*succ);
                    }
                }
            },
            Node::Exit {..} => continue,
        }
    }

    let pruned_nodes = nodes.into_iter()
        .filter(|(id, _)| visited.contains(id))
        .map(|(id,  mut node)| {
            let node = match node {
                Node::Entry { mut successors } => {
                    successors.retain(|succ| visited.contains(succ));
                    Node::Entry { successors }
                },
                Node::Exit { mut predecessors } => {
                    predecessors.retain(|pred| visited.contains(pred));
                    Node::Exit { predecessors }
                },
                Node::BasicBlock {
                    id,
                    mut predecessors,
                    mut successors,
                    insns,
                } => {
                    predecessors.retain(|pred| visited.contains(pred));
                    successors.retain(|succ| visited.contains(succ));
                    Node::BasicBlock {
                        id,
                        predecessors,
                        successors,
                        insns,
                    }
                }
            };
            (id, node)
        })
        .collect();

    Graph {
        nodes: pruned_nodes,
        label_map: cfg.label_map,
    }
}

fn remove_useless_jumps(cfg: Graph) -> Graph {
    let (ids, mut nodes) = cfg.nodes.into_iter()
        .map(|(id, node)| (id, node))
        .unzip::<NodeId, Node, Vec<_>, Vec<_>>();

    for i in 0..ids.len() - 1 {
        let id = ids[i];
        let default_succ = ids[i + 1];
        let node = &mut nodes[i];
        match node {
            Node::BasicBlock {
                insns,
                successors,
                ..
            } => {
                let last_insn = insns.last().unwrap();
                match last_insn {
                    Insn::Jump(..) |
                    Insn::BranchIfZero { .. } |
                    Insn::BranchNotZero { .. } => {
                        let mut keep_jump = false;
                        for succ in successors.iter() {
                            if *succ != default_succ {
                                keep_jump = true;
                                break;
                            }
                        }
                        if !keep_jump {
                            insns.pop();
                        }
                    },
                    _ => {}
                }
            }
            _ => {}
        }
    }

    let new_nodes = ids.into_iter()
        .zip(nodes)
        .collect();

    Graph {
        nodes: new_nodes,
        label_map: cfg.label_map,
    }
}

fn remove_useless_labels(cfg: Graph) -> Graph {
    let (ids, mut nodes) = cfg.nodes.into_iter()
        .map(|(id, node)| (id, node))
        .unzip::<NodeId, Node, Vec<_>, Vec<_>>();

    for i in 1..ids.len() {
        let id = ids[i];
        let node = &mut nodes[i];
        let default_pred = ids[i - 1];
        match node {
            Node::BasicBlock { predecessors, insns, .. } => {
                let mut keep_labels = false;
                for pred in predecessors.iter() {
                    if *pred != default_pred {
                        keep_labels = true;
                        break;
                    }
                }
                if !keep_labels {
                    // remove all labels at the beginning of the block
                    // this is an inefficient way to do it, but it works for now
                    loop {
                        if let Some(Insn::Label(_)) = insns.first() {
                            insns.remove(0);
                        } else {
                            break;
                        }
                    }
                }
            }
            _ => {}
        }
    }

    let new_nodes = ids.into_iter()
        .zip(nodes)
        .filter(|(_, node)| !matches!(node, Node::Exit { .. }))
        .collect();

    Graph {
        nodes: new_nodes,
        label_map: cfg.label_map,
    }
}