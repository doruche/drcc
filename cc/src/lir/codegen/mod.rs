
mod parse;
mod canonic;
mod regalloc;

use std::{collections::HashMap, marker::PhantomData};

use crate::common::*;
use super::{
    TopLevel,
    Function,
    StaticVar,
    DataSegment,
    BssSegment,
    Operand,
    Insn,
};

#[derive(Debug)]
pub struct CodeGen<Stage = Parse> {
    pub func_cxs: HashMap<StrDescriptor, FuncContext>,
    pub cur_func: Option<StrDescriptor>,
    pub _stage: PhantomData<Stage>,
}

#[derive(Debug, Clone)]
pub struct FuncContext {
    pub name: StrDescriptor,
    pub type_: FuncType,
    pub next_v_reg: usize,
    pub frame_size: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Parse;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Canonic;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RegAlloc;

impl CodeGen {
    pub fn new() -> Self {
        CodeGen {
            func_cxs: HashMap::new(),
            cur_func: None,
            _stage: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_inner() {
        todo!()
    }
}