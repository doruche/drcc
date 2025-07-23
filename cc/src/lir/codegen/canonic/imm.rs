//! Immediates-related canonicalization
//! e.g.    mv  t0, 1
//! ->      li  t0, 1
use crate::{asm::Register, common::*};
use super::{
    CodeGen,
    Insn,
    Canonic,
    Operand,
    LabelOperand,
    IntermediateInsn,
};

impl CodeGen<Canonic> {
    pub(super) fn canonic_imm(
        &mut self,
        insn: Insn,
    ) -> Vec<Insn> {
        use Insn::*;

        let mut insns = vec![];
    
        // we could do constant folding here,
        // but we've done once time during tac optimization.
        // we're not building a high-performance compiler,
        // so clear code overweighs performance for now.
        match insn {
            Sextw(Operand::Imm(_), Operand::Imm(_)) => {
                // this looks like a weird instruction,
                // but it exists cz we had done some type conversions
                // during tac generation.
                // in lir, this is not needed anymore.
                ;
            },
            Mv(dst, Operand::Imm(val)) |
            Sextw(dst, Operand::Imm(val)) =>
                insns.push(Insn::Li(dst, val)),
            Add(dst, Operand::Imm(left), Operand::Imm(right)) |
            Addw(dst, Operand::Imm(left), Operand::Imm(right)) => {
                insns.push(Insn::Li(Operand::PhysReg(Register::T5), left));
                match insn {
                    Add(..) => insns.push(Insn::Addi(dst, Operand::PhysReg(Register::T5), right)),
                    Addw(..) => insns.push(Insn::Addiw(dst, Operand::PhysReg(Register::T5), right as i32)),
                    _ => unreachable!(),
                }
            },
            Add(dst, left, right) |
            Addw(dst, left, right) |
            Sub(dst, left, right) |
            Subw(dst, left, right) |
            Mul(dst, left, right) |
            Mulw(dst, left, right) |
            Div(dst, left, right) |
            Divw(dst, left, right) |
            Rem(dst, left, right) |
            Remw(dst, left, right) |
            Slt(dst, left, right) |
            Sgt(dst, left, right) => {
                let (left, left_insn) = cimm_t5(left);
                let (right, right_insn) = cimm_t6(right);
                left_insn.map(|insn| insns.push(insn));
                right_insn.map(|insn| insns.push(insn));
                match insn {
                    Add(..) => insns.push(Insn::Add(dst, left, right)),
                    Addw(..) => insns.push(Insn::Addw(dst, left, right)),
                    Sub(..) => insns.push(Insn::Sub(dst, left, right)),
                    Subw(..) => insns.push(Insn::Subw(dst, left, right)),
                    Mul(..) => insns.push(Insn::Mul(dst, left, right)),
                    Mulw(..) => insns.push(Insn::Mulw(dst, left, right)),
                    Div(..) => insns.push(Insn::Div(dst, left, right)),
                    Divw(..) => insns.push(Insn::Divw(dst, left, right)),
                    Rem(..) => insns.push(Insn::Rem(dst, left, right)),
                    Remw(..) => insns.push(Insn::Remw(dst, left, right)),
                    Slt(..) => insns.push(Insn::Slt(dst, left, right)),
                    Sgt(..) => insns.push(Insn::Sgt(dst, left, right)),
                    _ => unreachable!(),
                }
            }
            Not(dst, src) |
            Neg(dst, src) |
            Negw(dst, src) |
            Seqz(dst, src) |
            Snez(dst, src) => {
                let (src, src_insn) = cimm_t5(src);
                src_insn.map(|insn| insns.push(insn));
                match insn {
                    Not(..) => insns.push(Insn::Not(dst, src)),
                    Neg(..) => insns.push(Insn::Neg(dst, src)),
                    Negw(..) => insns.push(Insn::Negw(dst, src)),
                    Seqz(..) => insns.push(Insn::Seqz(dst, src)),
                    Snez(..) => insns.push(Insn::Snez(dst, src)),
                    _ => unreachable!(),
                }
            },
            Beq(left, right, label) |
            Bne(left, right, label) =>{
                let (left, left_insn) = cimm_t5(left);
                let (right, right_insn) = cimm_t6(right);
                left_insn.map(|insn| insns.push(insn));
                right_insn.map(|insn| insns.push(insn));
                match insn {
                    Beq(..) => insns.push(Insn::Beq(left, right, label)),
                    Bne(..) => insns.push(Insn::Bne(left, right, label)),
                    _ => unreachable!(),
                }
            },
            Sd(src, mem) |
            Sw(src, mem) => {
                assert!(matches!(mem, Operand::Mem{..}));
                let (src, src_insn) = cimm_t5(src);
                src_insn.map(|insn| insns.push(insn));
                match insn {
                    Sd(..) => insns.push(Insn::Sd(src, mem)),
                    Sw(..) => insns.push(Insn::Sw(src, mem)),
                    _ => unreachable!(),
                }
            },
            Li(..) | La(..) => unreachable!(),
            Addi(..) | Addiw(..) |
            Call(..) |
            Label(..) |
            J(..) |
            Ret |
            Ld(..) |
            Lw(..) |
            LoadStatic(..) | 
            StoreStatic(..) |
            Intermediate(..) => {
                insns.push(insn);
            },
            _ => insns.push(insn),
        }

        insns
    }
}

fn cimm_t5(operand: Operand) -> (Operand, Option<Insn>) {
    match operand {
        Operand::Imm(val) => (Operand::PhysReg(Register::T5), Some(Insn::Li(Operand::PhysReg(Register::T5), val))),
        _ => (operand, None),
    }
}

fn cimm_t6(operand: Operand) -> (Operand, Option<Insn>) {
    match operand {
        Operand::Imm(val) => (Operand::PhysReg(Register::T6), Some(Insn::Li(Operand::PhysReg(Register::T6), val))),
        _ => (operand, None),
    }
}