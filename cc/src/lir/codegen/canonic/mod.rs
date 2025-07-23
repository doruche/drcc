//! We split the canonicalization pass into two parts:
//! 1.  flatten memory accesses
//! 2.  fix immediate values

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
    Canonic,
    CodeGen,
    Insn,
};

mod mem;
mod imm;

impl CodeGen<Canonic> {
    pub fn canonic(mut self, lir: TopLevel) -> TopLevel {
        let mut c_funcs = HashMap::new();

        for (name, func) in lir.functions {
            let func = self.canonic_func(func);
            c_funcs.insert(name, func);
        }
        
        TopLevel {
            functions: c_funcs,
            data_seg: lir.data_seg,
            bss_seg: lir.bss_seg,
            strtb: lir.strtb,
        }
    }

    fn canonic_func(
        &mut self,
        func: Function,
    ) -> Function {
        let mut func = func;
        self.cur_func = Some(func.name);

        let c_insns = func.body.into_iter()
            .flat_map(|insn| self.canonic_insn(insn))
            .collect();

        self.cur_func = None;

        func.body = c_insns;

        func
    }

    fn canonic_insn(
        &mut self,
        insn: Insn,
    ) -> Vec<Insn> {
        let cmem_insn = self.canonic_mem(insn);

        let cimm_insn = cmem_insn
            .into_iter()
            .flat_map(|insn| self.canonic_imm(insn))
            .collect();

        cimm_insn
    }
}