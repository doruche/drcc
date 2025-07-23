//! Canonicalization of the LIR code.

use std::collections::HashMap;

use crate::common::*;
use super::{
    CodeGen,
    Canonic,
    TopLevel,
    Function,
    StaticVar,
    DataSegment,
    BssSegment,
    Insn,
    Operand,
};

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
        self.cur_func = Some(func.name);

        let c_insns = func.body.into_iter()
            .filter_map(|insn| self.canonic_insn(insn))
            .flat_map(|insns| insns.into_iter())
            .collect();

        // align the frame size to 16 bytes
        let cx = self.cur_cx_mut();
        cx.frame_size = (cx.frame_size + 15) / 16 * 16;

        self.cur_func = None;

        Function {
            name: func.name,
            linkage: func.linkage,
            func_type: func.func_type,
            body: c_insns,
        }
    }

    fn canonic_insn(
        &mut self,
        insn: Insn,
    ) -> Option<Vec<Insn>> {
        todo!()
    }

    fn canonic_operand(
        &mut self,
        operand: Operand,
    ) -> (Operand, Option<Vec<Insn>>) {
        todo!()
    }
}