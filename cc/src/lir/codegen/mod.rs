mod parse;
mod canonic;
mod regalloc;

use std::{collections::HashMap, marker::PhantomData};

use crate::{common::*, tac::TacLabelOperand};
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
    // Map spilled virtual registers or variables to frame offsets
    pub mmap: HashMap<usize, isize>,
}

impl FuncContext {
    pub fn new(name: StrDescriptor, type_: FuncType) -> Self {
        FuncContext {
            name,
            type_,
            next_v_reg: 0,
            frame_size: 0,
            tmap: HashMap::new(),
            vmap: HashMap::new(),
            mmap: HashMap::new(),
        }
    }

    pub fn alloc_v_reg(&mut self) -> usize {
        let v_reg = self.next_v_reg;
        self.next_v_reg += 1;
        v_reg
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

    pub fn map_vreg2mem(&mut self, v_reg: usize, offset: isize) {
        assert!(self.mmap.insert(v_reg, offset).is_none(),
            "Virtual register {} already mapped to memory offset {}",
            v_reg, offset
        );
    }

    pub fn temp_vreg(&self, temp_id: usize) -> Option<usize> {
        self.tmap.get(&temp_id).copied()
    }

    pub fn var_vreg(&self, local_id: usize) -> Option<usize> {
        self.vmap.get(&local_id).copied()
    }

    pub fn alloc_frame(&mut self, size: usize) {
        self.frame_size += size;
    }
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
            next_label: 0,
            lmap: HashMap::new(),
            _stage: PhantomData,
        }
    }

    pub fn next_label(&mut self) -> usize {
        let label = self.next_label;
        self.next_label += 1;
        label
    }

    pub fn map_label(&mut self, signature: LabelSignature) -> usize {
        if let Some(&label_id) = self.lmap.get(&signature) {
            return label_id;
        }
        let label_id = self.next_label();
        self.lmap.insert(signature, label_id);
        label_id
    }
}