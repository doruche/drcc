//! Symbol definition and symbol table management

use std::collections::HashMap;

use crate::common::{DataType, Result, StrDescriptor};

#[derive(Debug, Clone, Copy)]
pub struct Symbol {
    id: SymbolId,
    inner: RawSymbol,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SymbolId(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RawSymbol {
    Variable {
        name: StrDescriptor,
        data_type: DataType,
    },
    Function {
        name: StrDescriptor,
        return_type: DataType,
        // params
    },
    // TypeName, for `typedef`, etc.
}

impl RawSymbol {
    pub fn name(&self) -> StrDescriptor {
        match self {
            RawSymbol::Variable { name, .. } => *name,
            RawSymbol::Function { name, .. } => *name,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SymbolTable {
    symbols: HashMap<SymbolId, Symbol>,
    next_id: usize,
}

#[derive(Debug)]
pub struct SymbolTableBuilder {

}

