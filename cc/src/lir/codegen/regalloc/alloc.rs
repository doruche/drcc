use std::{collections::HashMap, marker::PhantomData};

use crate::{asm::Register, common::*, lir::codegen::regalloc::{AnalyzeResult, GeneralReg, Rig}};
use super::{
    CodeGen,
    Spill,
    RegAlloc,
    TopLevel,
    Function,
    Insn,
    Operand,
};

impl CodeGen<RegAlloc> {
    pub fn alloc(mut self, lir: TopLevel) -> (TopLevel, CodeGen<Spill>) {
        let mut alloced_funcs = HashMap::new();

        for (name, func) in lir.functions {
            self.cur_func = Some(name);

            let mut func = self.alloc_func(func);

            func.callee_saved = self.cur_cx_mut()
                .callee_saved
                .take();

            alloced_funcs.insert(name, func);

            self.cur_func = None;
        }
 
        (TopLevel {
            functions: alloced_funcs,
            data_seg: lir.data_seg,
            bss_seg: lir.bss_seg,
            strtb: lir.strtb,
        }, CodeGen {
            func_cxs: self.func_cxs,
            cur_func: self.cur_func,
            next_label: self.next_label,
            lmap: self.lmap,
            _stage: PhantomData,
        })
    }

    pub fn alloc_func(
        &mut self,
        func: Function,
    ) -> Function {
        let mut func = func;
        let rig = Rig::base(&self.func_cxs);
        let AnalyzeResult {
            map,
        } = rig.analyze(&func);

        func.body = self.rewrite_insns(func.body, &map);

        func
    }

    fn rewrite_insns(
        &mut self,
        insns: Vec<Insn>,
        map: &HashMap<GeneralReg, Option<Register>>,
    ) -> Vec<Insn> {
        insns.into_iter()
            .map(|insn| self.rewrite_insn(insn, map))
            .collect()
    }

    fn rewrite_insn(
        &mut self,
        insn: Insn,
        map: &HashMap<GeneralReg, Option<Register>>,
    ) -> Insn {
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
            Insn::Sgt(dst, src1, src2) =>{
                let dst = self.rewrite_operand(dst, map);
                let src1 = self.rewrite_operand(src1, map);
                let src2 = self.rewrite_operand(src2, map);
                match insn {
                    Insn::Add(..) => Insn::Add(dst, src1, src2),
                    Insn::Addw(..) => Insn::Addw(dst, src1, src2),
                    Insn::Sub(..) => Insn::Sub(dst, src1, src2),
                    Insn::Subw(..) => Insn::Subw(dst, src1, src2),
                    Insn::Mul(..) => Insn::Mul(dst, src1, src2),
                    Insn::Mulw(..) => Insn::Mulw(dst, src1, src2),
                    Insn::Div(..) => Insn::Div(dst, src1, src2),
                    Insn::Divw(..) => Insn::Divw(dst, src1, src2),
                    Insn::Rem(..) => Insn::Rem(dst, src1, src2),
                    Insn::Remw(..) => Insn::Remw(dst, src1, src2),
                    Insn::Slt(..) => Insn::Slt(dst, src1, src2),
                    Insn::Sgt(..) => Insn::Sgt(dst, src1, src2),
                    _ => unreachable!(),
                }
            },
            Insn::Mv(dst, src) |
            Insn::Neg(dst, src) |
            Insn::Negw(dst, src) |
            Insn::Sextw(dst, src) |
            Insn::Seqz(dst, src) |
            Insn::Snez(dst, src) |
            Insn::Not(dst, src) => {
                let dst = self.rewrite_operand(dst, map);
                let src = self.rewrite_operand(src, map);
                match insn {
                    Insn::Mv(..) => Insn::Mv(dst, src),
                    Insn::Neg(..) => Insn::Neg(dst, src),
                    Insn::Negw(..) => Insn::Negw(dst, src),
                    Insn::Sextw(..) => Insn::Sextw(dst, src),
                    Insn::Seqz(..) => Insn::Seqz(dst, src),
                    Insn::Snez(..) => Insn::Snez(dst, src),
                    Insn::Not(..) => Insn::Not(dst, src),
                    _ => unreachable!(),
                }
            },
            Insn::Beq(src1, src2, label) |
            Insn::Bne(src1, src2, label) => {
                let src1 = self.rewrite_operand(src1, map);
                let src2 = self.rewrite_operand(src2, map);
                match insn {
                    Insn::Beq(..) => Insn::Beq(src1, src2, label),
                    Insn::Bne(..) => Insn::Bne(src1, src2, label),
                    _ => unreachable!(),
                }
            },
            Insn::Ret |
            Insn::Addi(..) |
            Insn::Addiw(..) |
            Insn::La(..) |
            Insn::Li(..) |
            Insn::Call(..) |
            Insn::Label(..) |
            Insn::J(..) |
            Insn::Intermediate(..) => insn,
            Insn::Ld(reg, mem) |
            Insn::Lw(reg, mem) |
            Insn::Sd(reg, mem) |
            Insn::Sw(reg, mem) => {
                let reg = self.rewrite_operand(reg, map);
                match insn {
                    Insn::Ld(..) => Insn::Ld(reg, mem),
                    Insn::Lw(..) => Insn::Lw(reg, mem),
                    Insn::Sd(..) => Insn::Sd(reg, mem),
                    Insn::Sw(..) => Insn::Sw(reg, mem),
                    _ => unreachable!(),
                }
            },
            Insn::LoadStatic(reg, name) |
            Insn::StoreStatic(reg, name) => {
                let reg = self.rewrite_operand(reg, map);
                match insn {
                    Insn::LoadStatic(..) => Insn::LoadStatic(reg, name),
                    Insn::StoreStatic(..) => Insn::StoreStatic(reg, name),
                    _ => unreachable!(),
                }
            }
        }
    }

    fn rewrite_operand(
        &mut self,
        operand: Operand,
        map: &HashMap<GeneralReg, Option<Register>>,
    ) -> Operand {
        match operand {
            Operand::Imm(..)|
            Operand::Mem{..}|
            Operand::PhysReg(..)|
            Operand::Static(..) => operand,
            Operand::VirtReg(vreg_id) => {
                let vreg = GeneralReg::Virt(vreg_id);
                if let Some(reg) = map.get(&vreg) {
                    if let Some(reg) = reg {
                        if reg.is_callee_saved() {
                            self.cur_cx_mut()
                                .push_callee_saved(*reg);
                        }

                        Operand::PhysReg(*reg)
                    } else {
                        Operand::VirtReg(vreg_id)
                    }
                } else {
                    Operand::VirtReg(vreg_id)
                }
            }
        }
    }
}