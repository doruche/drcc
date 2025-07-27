use crate::common::*;
use super::{
    CodeGen,
    Opt,
    Function,
    FuncContext,
    Insn,
    UnaryOp,
    BinaryOp,
    Operand,
};

impl CodeGen<Opt> {
    pub fn constant_folding(&mut self, func: Function) -> Function {
        
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
                let mut opted_body = vec![];
                
                for insn in body {
                    let opted_insn = match insn {
                        Insn::Unary { 
                            op, 
                            src, 
                            dst 
                        } => match (op, src, dst) {
                            (UnaryOp::Pos, ..) => None,
                            (UnaryOp::Negate, Operand::Imm(constant), dst) =>
                                Some(Insn::Move {
                                    src: Operand::Imm(constant.neg()),
                                    dst,
                                }),
                            (UnaryOp::Complement, Operand::Imm(constant), dst) => 
                                Some(Insn::Move {
                                    src: Operand::Imm(constant.complement()),
                                    dst,
                                }),
                            (UnaryOp::Not, Operand::Imm(constant), dst) =>
                                Some(Insn::Move {
                                    src: Operand::Imm(constant.not()),
                                    dst,
                                }),
                            _ => Some(Insn::Unary { op, src, dst }),
                        },
                        Insn::Binary {
                            op,
                            left,
                            right,
                            dst,
                        } => match (op, left, right, dst) {
                            (BinaryOp::Add, Operand::Imm(left), Operand::Imm(right), dst) => 
                                Some(Insn::Move {
                                    src: Operand::Imm(left + right),
                                    dst,
                                }),
                            (BinaryOp::Sub, Operand::Imm(left), Operand::Imm(right), dst) =>
                                Some(Insn::Move {
                                    src: Operand::Imm(left - right),
                                    dst,
                                }),
                            (BinaryOp::Mul, Operand::Imm(left), Operand::Imm(right), dst) =>
                                Some(Insn::Move {
                                    src: Operand::Imm(left * right),
                                    dst,
                                }),
                            (BinaryOp::Div, Operand::Imm(left), Operand::Imm(right), dst) => {
                                if right.is_zero() {
                                    Some(insn)     
                                } else {
                                    Some(Insn::Move {
                                        src: Operand::Imm(left / right),
                                        dst,
                                    })
                                }
                            },
                            (BinaryOp::Rem, Operand::Imm(left), Operand::Imm(right), dst) => {
                                if right.is_zero() {
                                    Some(insn)
                                } else {
                                    Some(Insn::Move {
                                        src: Operand::Imm(left % right),
                                        dst,
                                    })
                                }
                            },
                            (BinaryOp::Eq, Operand::Imm(left), Operand::Imm(right), dst) => 
                                Some(Insn::Move {
                                    src: Operand::Imm(if left == right { Constant::Int(1) } else { Constant::Int(0) }),
                                    dst,
                                }),
                            (BinaryOp::NotEq, Operand::Imm(left), Operand::Imm(right), dst) =>
                                Some(Insn::Move {
                                    src: Operand::Imm(if left != right { Constant::Int(1) } else { Constant::Int(0) }),
                                    dst,
                                }),
                            (BinaryOp::Ls, Operand::Imm(left), Operand::Imm(right), dst) =>
                                Some(Insn::Move {
                                    src: Operand::Imm(if left < right { Constant::Int(1) } else { Constant::Int(0) }),
                                    dst,
                                }),
                            (BinaryOp::LsEq, Operand::Imm(left), Operand::Imm(right), dst) =>
                                Some(Insn::Move {
                                    src: Operand::Imm(if left <= right { Constant::Int(1) } else { Constant::Int(0) }),
                                    dst,
                                }),
                            (BinaryOp::Gt, Operand::Imm(left), Operand::Imm(right), dst) =>
                                Some(Insn::Move {
                                    src: Operand::Imm(if left > right { Constant::Int(1) } else { Constant::Int(0) }),
                                    dst,
                                }),
                            (BinaryOp::GtEq, Operand::Imm(left), Operand::Imm(right), dst) =>
                                Some(Insn::Move {
                                    src: Operand::Imm(if left >= right { Constant::Int(1) } else { Constant::Int(0) }),
                                    dst,
                                }),
                            (BinaryOp::And, Operand::Imm(left), Operand::Imm(right), dst) =>
                                Some(Insn::Move {
                                    src: Operand::Imm(if left.is_zero() || right.is_zero() { Constant::Int(0) } else { Constant::Int(1) }),
                                    dst,
                                }),
                            (BinaryOp::Or, Operand::Imm(left), Operand::Imm(right), dst) =>
                                Some(Insn::Move {
                                    src: Operand::Imm(if left.is_zero() && right.is_zero() { Constant::Int(0) } else { Constant::Int(1) }),
                                    dst,
                                }),
                            _ => Some(Insn::Binary {
                                op,
                                left,
                                right,
                                dst,
                            }),
                        }
                        Insn::BranchIfZero { src, label } => {
                            if let Operand::Imm(constant) = src {
                                if constant.is_zero() {
                                    Some(Insn::Jump(label))
                                } else {
                                    None
                                }
                            } else {
                                Some(insn)
                            }
                        },
                        Insn::BranchNotZero { src, label } => {
                            if let Operand::Imm(constant) = src {
                                if constant.is_zero() {
                                    None
                                } else {
                                    Some(Insn::Jump(label))
                                }
                            } else {
                                Some(insn)
                            }
                        },
                        _ => Some(insn),
                    };
                    if let Some(insn) = opted_insn {
                        opted_body.push(insn);
                    }
                }
            
                Function::Defined {
                    return_type,
                    linkage,
                    name,
                    params,
                    local_vars,
                    body: opted_body,
                }
            }
        }
    }
}