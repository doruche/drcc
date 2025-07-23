use std::collections::HashMap;
use std::marker::PhantomData;

use crate::asm::riscv::TopLevel;
use crate::asm::Register;
use crate::{common::*, expect_mem, expect_register};
use crate::lir::{
    LirTopLevel,
    LirFunction,
    LirStaticVar,
    LirBssSegment,
    LirDataSegment,
    LirInsn,
    LirOperand,
    IntermediateInsn,
};
use super::{
    Insn,
    LabelOperand,
    CodeGen,
    Parse,
    Opt,
    Function,
    DataSegment,
    BssSegment,
    FuncContext,
};

impl CodeGen<Parse> {
    pub fn new() -> Self {
        CodeGen {
            cur_cx: None,
            _stage: PhantomData,
        }
    }

    pub fn parse(mut self, lir: LirTopLevel) -> (TopLevel, CodeGen<Opt>) {
        let mut functions = HashMap::new();
        let mut data_seg = DataSegment::new();
        let mut bss_seg = BssSegment::new();

        for (_name, var) in lir.data_seg.items.into_iter() {
            data_seg.add(var.into());
        }
        for (_name, var) in lir.bss_seg.items.into_iter() {
            bss_seg.add(var.into());
        }
        for (name, func) in lir.functions {
            let function = self.parse_func(func);
            functions.insert(name, function);
        }
    
        (TopLevel {
            functions,
            data_seg,
            bss_seg,
            strtb: lir.strtb,
        }, CodeGen { 
            cur_cx: None,
            _stage: PhantomData,
        })
    }

    fn parse_func(&mut self, func: LirFunction) -> Function {
        let cx = FuncContext {
            name: func.name,
            callee_saved: func.callee_saved.unwrap(),
            frame_size: func.frame_size,
        };
        self.cur_cx = Some(cx);

        let body = func.body.into_iter()
            .flat_map(|insn| self.parse_insn(insn))
            .collect();

        self.cur_cx = None;

        Function {
            name: func.name,
            func_type: func.func_type,
            body,
            linkage: func.linkage,
        }        
    }

    fn parse_insn(&mut self, insn: LirInsn) -> Vec<Insn> {
        let mut insns = vec![];

        match insn {
            LirInsn::Add(rd, rs1, rs2) |
            LirInsn::Addw(rd, rs1, rs2) |
            LirInsn::Sub(rd, rs1, rs2) |
            LirInsn::Subw(rd, rs1, rs2) |
            LirInsn::Mul(rd, rs1, rs2) |
            LirInsn::Mulw(rd, rs1, rs2) |
            LirInsn::Div(rd, rs1, rs2) |
            LirInsn::Divw(rd, rs1, rs2) |
            LirInsn::Rem(rd, rs1, rs2) |
            LirInsn::Remw(rd, rs1, rs2) |
            LirInsn::Slt(rd, rs1, rs2) |
            LirInsn::Sgt(rd, rs1, rs2) => {
                let rd = expect_register!(rd);
                let rs1 = expect_register!(rs1);
                let rs2 = expect_register!(rs2);
                insns.push(match insn {
                    LirInsn::Add(..) => Insn::Add(rd, rs1, rs2),
                    LirInsn::Addw(..) => Insn::Addw(rd, rs1, rs2),
                    LirInsn::Sub(..) => Insn::Sub(rd, rs1, rs2),
                    LirInsn::Subw(..) => Insn::Subw(rd, rs1, rs2),
                    LirInsn::Mul(..) => Insn::Mul(rd, rs1, rs2),
                    LirInsn::Mulw(..) => Insn::Mulw(rd, rs1, rs2),
                    LirInsn::Div(..) => Insn::Div(rd, rs1, rs2),
                    LirInsn::Divw(..) => Insn::Divw(rd, rs1, rs2),
                    LirInsn::Rem(..) => Insn::Rem(rd, rs1, rs2),
                    LirInsn::Remw(..) => Insn::Remw(rd, rs1, rs2),
                    LirInsn::Slt(..) => Insn::Slt(rd, rs1, rs2),
                    LirInsn::Sgt(..) => Insn::Sgt(rd, rs1, rs2),
                    _ => unreachable!(),
                });
            },
            LirInsn::Ld(dst, mem) => {
                let dst = expect_register!(dst);
                let (base, offset, size) = expect_mem!(mem);
                if size == 8 {
                    insns.push(Insn::Ld(dst, base, offset));
                } else { panic!("Internal error: expected memory operand of size 8") }
            },
            LirInsn::Lw(dst, mem) => {
                let dst = expect_register!(dst);
                let (base, offset, size) = expect_mem!(mem);
                if size == 4 {
                    insns.push(Insn::Lw(dst, base, offset));
                } else { panic!("Internal error: expected memory operand of size 4") }
            },
            LirInsn::Sd(src, mem) => {
                let src = expect_register!(src);
                let (base, offset, size) = expect_mem!(mem);
                if size == 8 {
                    insns.push(Insn::Sd(src, base, offset));
                } else { panic!("Internal error: expected memory operand of size 8") }
            },
            LirInsn::Sw(src, mem) => {
                let src = expect_register!(src);
                let (base, offset, size) = expect_mem!(mem);
                if size == 4 {
                    insns.push(Insn::Sw(src, base, offset));
                } else { panic!("Internal error: expected memory operand of size 4") }
            },
            LirInsn::Li(rd, imm) => {
                let rd = expect_register!(rd);
                insns.push(Insn::Li(rd, imm));
            },
            LirInsn::La(rd, name) => {
                let rd = expect_register!(rd);
                insns.push(Insn::La(rd, name));
            },
            LirInsn::Addi(rd, rs, imm) => {
                let rd = expect_register!(rd);
                let rs = expect_register!(rs);
                insns.push(Insn::Addi(rd, rs, imm));
            },
            LirInsn::Addiw(rd, rs, imm) => {
                let rd = expect_register!(rd);
                let rs = expect_register!(rs);
                insns.push(Insn::Addiw(rd, rs, imm));
            },
            LirInsn::Mv(dst, src) |
            LirInsn::Neg(dst, src) |
            LirInsn::Negw(dst, src) |
            LirInsn::Not(dst, src) |
            LirInsn::Seqz(dst, src) |
            LirInsn::Snez(dst, src) |
            LirInsn::Sextw(dst, src) => {
                let dst = expect_register!(dst);
                let src = expect_register!(src);
                insns.push(match insn {
                    LirInsn::Mv(..) => Insn::Mv(dst, src),
                    LirInsn::Neg(..) => Insn::Neg(dst, src),
                    LirInsn::Negw(..) => Insn::Negw(dst, src),
                    LirInsn::Not(..) => Insn::Not(dst, src),
                    LirInsn::Seqz(..) => Insn::Seqz(dst, src),
                    LirInsn::Snez(..) => Insn::Snez(dst, src),
                    LirInsn::Sextw(..) => Insn::Sextw(dst, src),
                    _ => unreachable!(),
                });
            },
            LirInsn::Beq(rs1, rs2, label) |
            LirInsn::Bne(rs1, rs2, label) => {
                let rs1 = expect_register!(rs1);
                let rs2 = expect_register!(rs2);
                insns.push(match insn {
                    LirInsn::Beq(..) => Insn::Beq(rs1, rs2, label.into()),
                    LirInsn::Bne(..) => Insn::Bne(rs1, rs2, label.into()),
                    _ => unreachable!(),
                });
            },
            LirInsn::J(label) => insns.push(Insn::J(label.into())),
            LirInsn::Label(label) => insns.push(Insn::Label(label.into())),
            LirInsn::LoadStatic(rd, namr) => insns.push(Insn::LoadStatic(expect_register!(rd), namr)),
            LirInsn::StoreStatic(rs, name) => insns.push(Insn::StoreStatic(expect_register!(rs), name)),
            LirInsn::Call(name) => insns.push(Insn::Call(name)),
            LirInsn::Intermediate(insn) => insns.extend(self.parse_intermediate(insn)),
            LirInsn::Ret => unreachable!(),
        }

        insns
    }

    fn parse_intermediate(&mut self, insn: IntermediateInsn) -> Vec<Insn> {
        let cx = self.cur_cx();
        let mut insns = vec![];

        match insn {
            IntermediateInsn::Prologue => {
                insns.push(Insn::Addi(Register::Sp, Register::Sp, cx.frame_size as i64));
                insns.push(Insn::Addi(Register::S0, Register::Sp, -(cx.frame_size as i64)));
                for &(reg, offset) in cx.callee_saved.iter() {
                    insns.push(Insn::Sd(reg, Register::S0, offset));
                }
            },
            IntermediateInsn::Epilogue => {
                for &(reg, offset) in cx.callee_saved.iter().rev() {
                    insns.push(Insn::Ld(reg, Register::S0, offset));
                }
                insns.push(Insn::Addi(Register::Sp, Register::Sp, -(cx.frame_size as i64)));
                insns.push(Insn::Ret);
            },
        }

        insns
    }

    fn cur_cx(&self) -> &FuncContext {
        self.cur_cx.as_ref().expect("No current context set")
    }
}

