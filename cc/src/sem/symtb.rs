//! Symbol definition and symbol table management

use std::{collections::HashMap};

use crate::common:: {
    DataType, Error, StrDescriptor
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VarSymbol {
    pub(super) name: StrDescriptor,
    pub(super) type_: DataType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FuncSymbol {
    pub(super) name: StrDescriptor,
    pub(super) return_type: DataType,
    // params
    // todo
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymError {
    DuplicateDecl(StrDescriptor),
    NotFound(StrDescriptor),
    InvalidLValue,
}


// We have to split namespaces for different symbol types,
// as C does allow same names for different symbol types.

#[derive(Debug, Clone)]
pub struct SymbolTable {
    var_ns: Vec<HashMap<StrDescriptor, VarSymbol>>,
    func_ns: HashMap<StrDescriptor, FuncSymbol>,    // C does not allow nested functions.
    // func_ns
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            var_ns: vec![HashMap::new()],
            func_ns: HashMap::new(),
            // func_ns: vec![],
        }
    }

    pub fn enter_scope(&mut self) {
        self.var_ns.push(HashMap::new());
    }

    pub fn exit_scope(&mut self) {
        if self.var_ns.len() == 1 {
            panic!("Cannot exit global scope");
        }
        assert!(self.var_ns.pop().is_some());
    }

    pub fn def_var(
        &mut self, 
        name: StrDescriptor, 
        type_: DataType
    ) -> Result<(), SymError> {
        let cur_scope = self.var_ns.last_mut()
            .ok_or(SymError::NotFound(name))?;
        if cur_scope.contains_key(&name) {
            return Err(SymError::DuplicateDecl(name));
        }

        cur_scope.insert(name, VarSymbol { name, type_ });
        Ok(())
    }

    pub fn def_func(
        &mut self, 
        name: StrDescriptor, 
        return_type: DataType,
    ) -> Result<(), SymError> {
        if self.func_ns.contains_key(&name) {
            return Err(SymError::DuplicateDecl(name));
        }
        self.func_ns.insert(name, FuncSymbol { name, return_type });
        Ok(())
    }

    pub fn lookup_func(
        &self, 
        name: StrDescriptor
    ) -> Result<FuncSymbol, SymError> {
        self.func_ns.get(&name)
            .copied()
            .ok_or(SymError::NotFound(name))
    }

    pub fn lookup_var(
        &self, 
        name: StrDescriptor
    ) -> Result<VarSymbol, SymError> {
        for scope in self.var_ns.iter().rev() {
            if let Some(symbol) = scope.get(&name) {
                return Ok(*symbol);
            }
        }
        Err(SymError::NotFound(name))
    }

    pub fn lookup_var_cur(
        &self, 
        name: StrDescriptor
    ) -> Result<VarSymbol, SymError> {
        if let Some(symbol) = self.var_ns.last()
            .and_then(|scope| scope.get(&name)) {
            Ok(*symbol)
        } else {
            Err(SymError::NotFound(name))
        }
    }
}



