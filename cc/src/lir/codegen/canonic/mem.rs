//! Memory access canonicalization.
//! e.g.    add t0, t1, 4(t2)
//! ->      ld  t2, 4(t2)
//! ->      add t0, t1, t2
//! We use t5&t6 as temporary registers for memory access.

use crate::{asm::Register, common::*};
use super::{
    TopLevel,
    Function,
    Operand,
    Insn,
    CodeGen,
    Canonic,
};

impl CodeGen<Canonic> {
    pub(super) fn canonic_mem(
        &mut self,
        insn: Insn,
    ) -> Vec<Insn> {
        let mut insns = vec![];

        match insn {
            Insn::Add(dst, left, right) |
            Insn::Addw(dst, left, right) |
            Insn::Sub(dst, left, right) |
            Insn::Subw(dst, left, right) |
            Insn::Mul(dst, left, right) |
            Insn::Mulw(dst, left, right) |
            Insn::Div(dst, left, right) |
            Insn::Divw(dst, left, right) |
            Insn::Rem(dst, left, right) |
            Insn::Remw(dst, left, right) |
            Insn::Slt(dst, left, right) |
            Insn::Sgt(dst, left, right) => {
                let (left, left_insn) = cmem_r_t5(left);
                let (right, right_insn) = cmem_r_t6(right);
                let (dst, dst_insn) = cmem_w_t5(dst);

                if let Some(insn) = left_insn {
                    insns.push(insn);
                }
                if let Some(insn) = right_insn {
                    insns.push(insn);
                }
                
                insns.push(match insn {
                    Insn::Add(..) => Insn::Add(dst, left, right),
                    Insn::Addw(..) => Insn::Addw(dst, left, right),
                    Insn::Sub(..) => Insn::Sub(dst, left, right),
                    Insn::Subw(..) => Insn::Subw(dst, left, right),
                    Insn::Mul(..) => Insn::Mul(dst, left, right),
                    Insn::Mulw(..) => Insn::Mulw(dst, left, right),
                    Insn::Div(..) => Insn::Div(dst, left, right),
                    Insn::Divw(..) => Insn::Divw(dst, left, right),
                    Insn::Rem(..) => Insn::Rem(dst, left, right),
                    Insn::Remw(..) => Insn::Remw(dst, left, right),
                    Insn::Slt(..) => Insn::Slt(dst, left, right),
                    Insn::Sgt(..) => Insn::Sgt(dst, left, right),
                    _ => unreachable!(),
                });
                
                if let Some(insn) = dst_insn {
                    insns.push(insn);
                }
            },
            Insn::Beq(left, right, label) |
            Insn::Bne(left, right, label) => {
                let (left, left_insn) = cmem_r_t5(left);
                let (right, right_insn) = cmem_r_t6(right);

                if let Some(insn) = left_insn {
                    insns.push(insn);
                }
                if let Some(insn) = right_insn {
                    insns.push(insn);
                }

                insns.push(match insn {
                    Insn::Beq(..) => Insn::Beq(left, right, label),
                    Insn::Bne(..) => Insn::Bne(left, right, label),
                    _ => unreachable!(),
                });
            },
            Insn::Mv(dst, src) => {
                let (src, src_insn) = cmem_r_t5(src);
                let (dst, dst_insn) = cmem_w_t5(dst);

                if let Some(insn) = src_insn {
                    insns.push(insn);
                }
                
                if src_insn.is_none() || dst_insn.is_none() {
                    insns.push(Insn::Mv(dst, src));
                } else {
                    ;
                }

                if let Some(insn) = dst_insn {
                    insns.push(insn);
                }
            },
            Insn::LoadStatic(dst, name) => {
                let (dst, dst_insn) = cmem_w_t5(dst);

                insns.push(Insn::LoadStatic(dst, name));
                
                if let Some(insn) = dst_insn {
                    insns.push(insn);
                }

            },
            Insn::StoreStatic(src, name) => {
                let (src, src_insn) = cmem_r_t5(src);

                if let Some(insn) = src_insn {
                    insns.push(insn);
                }

                insns.push(Insn::StoreStatic(src, name));
            },
            Insn::Ld(dst, mem) |
            Insn::Lw(dst, mem) => {
                assert!(matches!(mem, Operand::Mem{..}));
                let (dst, dst_insn) = cmem_w_t5(dst);

                insns.push(match insn {
                    Insn::Ld(..) => Insn::Ld(dst, mem),
                    Insn::Lw(..) => Insn::Lw(dst, mem),
                    _ => unreachable!(),
                });

                if let Some(insn) = dst_insn {
                    insns.push(insn);
                }
            },
            Insn::Sd(src, mem) |
            Insn::Sw(src, mem) => {
                assert!(matches!(mem, Operand::Mem{..}));
                let (src, src_insn) = cmem_r_t5(src);

                if let Some(insn) = src_insn {
                    insns.push(insn);
                }

                insns.push(match insn {
                    Insn::Sd(..) => Insn::Sd(src, mem),
                    Insn::Sw(..) => Insn::Sw(src, mem),
                    _ => unreachable!(),
                });
            },
            Insn::Neg(dst, src) |
            Insn::Negw(dst, src) |
            Insn::Not(dst, src) |
            Insn::Seqz(dst, src) |
            Insn::Snez(dst, src) |
            Insn::Sextw(dst, src) => {
                let (src, src_insn) = cmem_r_t5(src);
                let (dst, dst_insn) = cmem_w_t5(dst);

                if let Some(insn) = src_insn {
                    insns.push(insn);
                }

                insns.push(match insn {
                    Insn::Neg(..) => Insn::Neg(dst, src),
                    Insn::Negw(..) => Insn::Negw(dst, src),
                    Insn::Not(..) => Insn::Not(dst, src),
                    Insn::Seqz(..) => Insn::Seqz(dst, src),
                    Insn::Snez(..) => Insn::Snez(dst, src),
                    Insn::Sextw(..) => Insn::Sextw(dst, src),
                    _ => unreachable!(),
                });

                if let Some(insn) = dst_insn {
                    insns.push(insn);
                }
            },
            Insn::Call(..) |
            Insn::Ret |
            Insn::Label(..) |
            Insn::J(..) |
            Insn::Addi(..) |
            Insn::Addiw(..) |
            Insn::Intermediate(..) => {
                insns.push(insn);
            },
            Insn::Li(..) | Insn::La(..) => unreachable!(),
        }

        insns
    }
}

fn cmem_r_t5(operand: Operand) -> (Operand, Option<Insn>) {
    match operand {
        Operand::Mem { base, offset, size } => (
            Operand::PhysReg(Register::T5),
            match size {
                4 => Some(Insn::Lw(Operand::PhysReg(Register::T5), Operand::Mem { base, offset, size })),
                8 => Some(Insn::Ld(Operand::PhysReg(Register::T5), Operand::Mem { base, offset, size })),
                _ => unreachable!(),
            }  
        ),
        Operand::Static(name) => (
            Operand::PhysReg(Register::T5),
            Some(Insn::LoadStatic(Operand::PhysReg(Register::T5), name)),
        ),
        _ => (operand, None),
    }
}

fn cmem_r_t6(operand: Operand) -> (Operand, Option<Insn>) {
    match operand {
        Operand::Mem { base, offset, size } => (
            Operand::PhysReg(Register::T6),
            match size {
                4 => Some(Insn::Lw(Operand::PhysReg(Register::T6), Operand::Mem { base, offset, size })),
                8 => Some(Insn::Ld(Operand::PhysReg(Register::T6), Operand::Mem { base, offset, size })),
                _ => unreachable!(),
            }
        ),
        Operand::Static(name) => (
            Operand::PhysReg(Register::T6),
            Some(Insn::LoadStatic(Operand::PhysReg(Register::T6), name)),
        ),
        _ => (operand, None),
    }
}

fn cmem_w_t5(operand: Operand) -> (Operand, Option<Insn>) {
    match operand {
        Operand::Mem {
            base,
            offset,
            size,
        } => (
            Operand::PhysReg(Register::T5),
            match size {
                4 => Some(Insn::Sw(Operand::PhysReg(Register::T5), Operand::Mem { base, offset, size })),
                8 => Some(Insn::Sd(Operand::PhysReg(Register::T5), Operand::Mem { base, offset, size })),
                _ => unreachable!(),
            }
        ),
        Operand::Static(name) => (
            Operand::PhysReg(Register::T5),
            Some(Insn::StoreStatic(Operand::PhysReg(Register::T5), name))
        ),
        _ => (operand, None),
    }
}