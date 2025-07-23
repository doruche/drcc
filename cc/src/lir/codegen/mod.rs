mod parse;
mod regalloc;
mod spill;
mod canonic;

use std::{collections::HashMap, marker::PhantomData};

use crate::{asm::Register, common::*, tac::TacLabelOperand};
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
};

#[derive(Debug)]
pub struct CodeGen<Stage = Parse> {
    pub func_cxs: HashMap<StrDescriptor, FuncContext>,
    pub cur_func: Option<StrDescriptor>,
    
    pub next_label: usize,
    pub lmap: HashMap<LabelSignature, usize>,

    pub _stage: PhantomData<Stage>,
}

#[derive(Debug, Clone)]
pub struct FuncContext {
    pub name: StrDescriptor,
    pub type_: FuncType,

    // following fields will be used through all stages.
    pub next_v_reg: usize,
    pub frame_size: usize,
    // Map temporary variables' id to virtual registers
    pub tmap: HashMap<usize, usize>,
    // Map variables' local id to virtual registers
    pub vmap: HashMap<usize, usize>,
    // Map spilled virtual registers to frame offsets
    pub mmap: HashMap<usize, isize>,
    // registers that need to be saved across function calls
    // (register, frame_offset)
    pub callee_saved: Option<Vec<(Register, isize)>>,
}

impl FuncContext {
    pub fn new(name: StrDescriptor, type_: FuncType) -> Self {
        FuncContext {
            name,
            type_,
            next_v_reg: 0,
            frame_size: 16,
            tmap: HashMap::new(),
            vmap: HashMap::new(),
            mmap: HashMap::new(),
            callee_saved: Some(vec![(Register::S0, -16), (Register::Ra, -8)]),
        }
    }

    pub fn alloc_v_reg(&mut self) -> usize {
        let v_reg = self.next_v_reg;
        self.next_v_reg += 1;
        v_reg
    }

    pub fn map_vreg2frame(&mut self, v_reg: usize, offset: isize) {
        assert!(self.mmap.insert(v_reg, offset).is_none(),
            "Virtual register {} already mapped to memory offset {}",
            v_reg, offset
        );
    }

    pub fn map_temp2vreg(&mut self, temp_id: usize, v_reg: usize) {
        assert!(self.tmap.insert(temp_id, v_reg).is_none(),
            "Temporary variable with id {} already mapped to v_reg {}",
            temp_id, v_reg
        );
    }

    pub fn map_var2vreg(&mut self, local_id: usize, v_reg: usize) {
        assert!(self.vmap.insert(local_id, v_reg).is_none(),
            "Variable with local id {} already mapped to v_reg {}",
            local_id, v_reg
        );
    }

    pub fn temp_vreg(&self, temp_id: usize) -> Option<usize> {
        self.tmap.get(&temp_id).copied()
    }

    pub fn var_vreg(&self, local_id: usize) -> Option<usize> {
        self.vmap.get(&local_id).copied()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Parse;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RegAlloc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Spill;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Canonic;


impl CodeGen {
    pub fn new() -> Self {
        CodeGen {
            func_cxs: HashMap::new(),
            cur_func: None,
            next_label: 0,
            lmap: HashMap::new(),
            _stage: PhantomData,
        }
    }
}

impl<Stage> CodeGen<Stage> {
    fn cur_cx(&self) -> &FuncContext {
        self.cur_func
            .as_ref()
            .and_then(|name| self.func_cxs.get(name))
            .expect("Internal error: Current function context not found")
    }

    fn cur_cx_mut(&mut self) -> &mut FuncContext {
        self.cur_func
            .as_ref()
            .and_then(|name| self.func_cxs.get_mut(name))
            .expect("Internal error: Current function context not found")
    }
}