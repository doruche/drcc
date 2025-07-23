use std::{collections::HashMap, marker::PhantomData};

use crate::common::*;
use super::{
    TopLevel,
    Function,
    StaticVar,
    DataSegment,
    BssSegment,
    Operand,
    LabelOperand,
    LabelSignature,
    IntermediateInsn,
    Insn,
    CodeGen,
    Canonic,
    Spill,
};

impl CodeGen<Spill> {
    pub fn spill(mut self, lir: TopLevel) -> (TopLevel, CodeGen<Canonic>) {
        let mut s_funcs = HashMap::new();
        for (name, func) in lir.functions {
            let func = self.spill_func(func);
            s_funcs.insert(name, func);
        }
        (TopLevel {
            functions: s_funcs,
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

    fn spill_func(
        &mut self,
        func: Function,
    ) -> Function {
        let mut func = func;
        self.cur_func = Some(func.name);

        let s_insns = func.body.into_iter()
            .map(|insn| self.spill_insn(insn))
            .collect();

        // align the frame size to 16 bytes
        let cx = self.cur_cx_mut();
        cx.frame_size = (cx.frame_size + 15) / 16 * 16;

        // We won't further use these fields in FuncContext anymore.
        func.body = s_insns;
        func.frame_size = cx.frame_size;
        func.callee_saved = cx.callee_saved.take();


        self.cur_func = None;

        func        
    }

    fn spill_insn(
        &mut self,
        insn: Insn,
    ) -> Insn {
        match insn {
            Insn::Intermediate(_) => insn,
            Insn::Add(dst, left, right) => {
                let dst = self.spill_operand(dst, 8);
                let left = self.spill_operand(left, 8);
                let right = self.spill_operand(right, 8);
                Insn::Add(dst, left, right)
            },
            Insn::Addw(dst, left, right) => {
                let dst = self.spill_operand(dst, 4);
                let left = self.spill_operand(left, 4);
                let right = self.spill_operand(right, 4);
                Insn::Addw(dst, left, right)
            },
            Insn::Addi(dst, src, imm) => {
                let dst = self.spill_operand(dst, 8);
                let src = self.spill_operand(src, 8);
                Insn::Addi(dst, src, imm)
            },
            Insn::Sub(dst, left, right) => {
                let dst = self.spill_operand(dst, 8);
                let left = self.spill_operand(left, 8);
                let right = self.spill_operand(right, 8);
                Insn::Sub(dst, left, right)
            },
            Insn::Subw(dst, left, right) => {
                let dst = self.spill_operand(dst, 4);
                let left = self.spill_operand(left, 4);
                let right = self.spill_operand(right, 4);
                Insn::Subw(dst, left, right)
            },
            Insn::Mul(dst, left, right) => {
                let dst = self.spill_operand(dst, 8);
                let left = self.spill_operand(left, 8);
                let right = self.spill_operand(right, 8);
                Insn::Mul(dst, left, right)
            },
            Insn::Mulw(dst, left, right) => {
                let dst = self.spill_operand(dst, 4);
                let left = self.spill_operand(left, 4);
                let right = self.spill_operand(right, 4);
                Insn::Mulw(dst, left, right)
            },
            Insn::Div(dst, left, right) => {
                let dst = self.spill_operand(dst, 8);
                let left = self.spill_operand(left, 8);
                let right = self.spill_operand(right, 8);
                Insn::Div(dst, left, right)
            },
            Insn::Divw(dst, left, right) => {
                let dst = self.spill_operand(dst, 4);
                let left = self.spill_operand(left, 4);
                let right = self.spill_operand(right, 4);
                Insn::Divw(dst, left, right)
            },
            Insn::Rem(dst, left, right) => {
                let dst = self.spill_operand(dst, 8);
                let left = self.spill_operand(left, 8);
                let right = self.spill_operand(right, 8);
                Insn::Rem(dst, left, right)
            },
            Insn::Remw(dst, left, right) => {
                let dst = self.spill_operand(dst, 4);
                let left = self.spill_operand(left, 4);
                let right = self.spill_operand(right, 4);
                Insn::Remw(dst, left, right)
            },
            Insn::Beq(left, right, label) => {
                let left = self.spill_operand(left, 8);
                let right = self.spill_operand(right, 8);
                Insn::Beq(left, right, label)
            },
            Insn::Bne(left, right, label) => {
                let left = self.spill_operand(left, 8);
                let right = self.spill_operand(right, 8);
                Insn::Bne(left, right, label)
            },
            Insn::Slt(dst, left, right) => {
                let dst = self.spill_operand(dst, 8);
                let left = self.spill_operand(left, 8);
                let right = self.spill_operand(right, 8);
                Insn::Slt(dst, left, right)
            },
            Insn::Sgt(dst, left, right) => {
                let dst = self.spill_operand(dst, 8);
                let left = self.spill_operand(left, 8);
                let right = self.spill_operand(right, 8);
                Insn::Sgt(dst, left, right)
            },
            Insn::Seqz(dst, src) => {
                let dst = self.spill_operand(dst, 8);
                let src = self.spill_operand(src, 8);
                Insn::Seqz(dst, src)
            },
            Insn::Snez(dst, src) => {
                let dst = self.spill_operand(dst, 8);
                let src = self.spill_operand(src, 8);
                Insn::Snez(dst, src)
            },
            Insn::Sextw(dst, src) => {
                let dst = self.spill_operand(dst, 8);
                let src = self.spill_operand(src, 4);
                Insn::Sextw(dst, src)
            },
            Insn::Label(label) => Insn::Label(label),
            Insn::Call(name) => Insn::Call(name),
            Insn::Ret => Insn::Ret,
            Insn::J(label) => Insn::J(label),
            Insn::Ld(dst, mem) => {
                let dst = self.spill_operand(dst, 8);
                let mem = self.spill_operand(mem, 8);
                Insn::Ld(dst, mem)
            },
            Insn::Lw(dst, mem) => {
                let dst = self.spill_operand(dst, 4);
                let mem = self.spill_operand(mem, 4);
                Insn::Lw(dst, mem)
            },
            Insn::Sd(src, mem) => {
                let src = self.spill_operand(src, 8);
                let mem = self.spill_operand(mem, 8);
                Insn::Sd(src, mem)
            },
            Insn::Sw(src, mem) => {
                let src = self.spill_operand(src, 4);
                let mem = self.spill_operand(mem, 4);
                Insn::Sw(src, mem)
            },
            Insn::Mv(dst, src) => {
                let dst = self.spill_operand(dst, 8);
                let src = self.spill_operand(src, 8);
                Insn::Mv(dst, src)
            },
            Insn::Neg(dst, src) => {
                let dst = self.spill_operand(dst, 8);
                let src = self.spill_operand(src, 8);
                Insn::Neg(dst, src)
            },
            Insn::Negw(dst, src) => {
                let dst = self.spill_operand(dst, 4);
                let src = self.spill_operand(src, 4);
                Insn::Negw(dst, src)
            },
            Insn::Not(dst, src) => {
                let dst = self.spill_operand(dst, 8);
                let src = self.spill_operand(src, 8);
                Insn::Not(dst, src)
            },
            _ => unreachable!(),
        }
    }

    fn spill_operand(
        &mut self,
        operand: Operand,
        size: usize,
    ) -> Operand {
        match operand {
            Operand::VirtReg(v_reg_id) => {
                let offset = self.spill_vreg(v_reg_id);
                Operand::frame(offset, size)
            },
            _ => operand,
        }
    }
}

impl CodeGen<Spill> {
    fn spill_vreg(
        &mut self,
        v_reg: usize,
    ) -> isize {
        let cx = self.cur_cx_mut();
        if let Some(&offset) = cx.mmap.get(&v_reg) {
            return offset;
        } else {
            let offset = -(cx.frame_size as isize + 8);
            cx.frame_size += 8;
            cx.map_vreg2frame(v_reg, offset);
            return offset;
        }
    }
}