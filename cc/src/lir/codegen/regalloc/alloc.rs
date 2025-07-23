use std::marker::PhantomData;

use crate::common::*;
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
        (lir, CodeGen {
            func_cxs: self.func_cxs,
            cur_func: self.cur_func,
            next_label: self.next_label,
            lmap: self.lmap,
            _stage: PhantomData,
        })
    }
}