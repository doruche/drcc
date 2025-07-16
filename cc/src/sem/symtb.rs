//! Symbol definition and symbol table management

use std::{collections::HashMap, fmt::Debug};

use crate::{common:: {
    DataType, Error, FuncType, Span, StrDescriptor, StringPool, Linkage,
}, sem::hir::Param};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VarSymbol {
    pub(super) name: StrDescriptor,
    pub(super) type_: DataType,
    pub(super) linkage: Linkage,
}
 
#[derive(Debug, Clone)]
pub struct FuncSymbol {
    pub(super) name: StrDescriptor,
    pub(super) type_: FuncType,
    pub(super) linkage: Linkage,
    pub(super) is_definition: bool,
}

/// In the first pass - name resolution - we only need to know
/// the name of the symbol, not its type.
/// Whether types are matched will be determined in the type checking pass.
#[derive(Debug, Clone)]
pub enum CommonSymbol {
    Var(StrDescriptor),
    Func(StrDescriptor),
}

impl CommonSymbol {
    pub fn name(&self) -> StrDescriptor {
        match self {
            CommonSymbol::Var(name) => *name,
            CommonSymbol::Func(name) => *name,
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LabelSymbol {
    pub(super) name: StrDescriptor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymError {
    DuplicateDecl(StrDescriptor),
    VarNotFound(StrDescriptor),
    FuncNotFound(StrDescriptor),
    LabelNotFound(StrDescriptor),
    FuncRedefinition(StrDescriptor),
    FuncDefNotGlobal(StrDescriptor),
    TypeMismatch {
        expected: DataType,
        found: DataType,
    },
    FuncTypeMismatch(StrDescriptor),
    InvalidLValue,
}

impl SymError {
    pub fn to_error(self, strtb: &StringPool, span: Span) -> Error {
        match self {
            SymError::DuplicateDecl(sd) => Error::semantic(
                format!("Symbol '{}' is already defined in this scope.", strtb.get(sd).unwrap()),
                span,
            ),
            SymError::VarNotFound(sd) => Error::semantic(
                format!("Variable '{}' is not defined.", strtb.get(sd).unwrap()),
                span,
            ),
            SymError::FuncNotFound(sd) => Error::semantic(
                format!("Function '{}' is not defined.", strtb.get(sd).unwrap()),
                span,
            ),
            SymError::LabelNotFound(sd) => Error::semantic(
                format!("Label '{}' is not defined.", strtb.get(sd).unwrap()),
                span,
            ),
            SymError::FuncRedefinition(sd) => Error::semantic(
                format!("Function '{}' is already defined.", strtb.get(sd).unwrap()),
                span,
            ),
            SymError::FuncDefNotGlobal(sd) => Error::semantic(
                format!("Function '{}' is defined in a non-global scope.", strtb.get(sd).unwrap()),
                span,
            ),
            SymError::TypeMismatch { expected, found } => Error::semantic(
                format!("Type mismatch: expected {:?}, found {:?}.", expected, found),
                span,
            ),
            SymError::FuncTypeMismatch(name) => Error::semantic(
                format!("Function '{}' has a type mismatch.", strtb.get(name).unwrap()),
                span,
            ),
            SymError::InvalidLValue => Error::semantic(
                "Left-hand side of assignment must be a variable.".to_string(),
                span,
            ),
        }
    }
}


#[derive(Debug, Clone)]
pub struct SymbolTable {
    common_ns: Vec<HashMap<StrDescriptor, CommonSymbol>>,
    func_defs: HashMap<StrDescriptor, FuncSymbol>,
    label_ns: HashMap<StrDescriptor, usize>,
}


// Common methods for the symbol table
impl SymbolTable {
    pub fn new() -> Self {
        Self {
            common_ns: vec![HashMap::new()],
            func_defs: HashMap::new(),
            label_ns: HashMap::new(),
        }
    }

    /// This must be called before any function definition.
    pub fn clear_labels(&mut self) {
        self.label_ns.clear();
    }

    pub fn def_label(&mut self, name: StrDescriptor) -> Result<(), SymError> {
        if self.label_ns.contains_key(&name) {
            return Err(SymError::DuplicateDecl(name));
        }
        let index = self.label_ns.len();
        self.label_ns.insert(name, index);
        Ok(())
    }

    pub fn lookup_label(&self, name: StrDescriptor) -> Result<usize, SymError> {
        self.label_ns.get(&name)
            .copied()
            .ok_or(SymError::LabelNotFound(name))
    }

    pub fn enter_block(&mut self) {
        self.common_ns.push(HashMap::new());
    }

    pub fn exit_block(&mut self) {
        if self.common_ns.len() == 1 {
            panic!("Cannot exit global scope");
        }
        assert!(self.common_ns.pop().is_some());
    }

    pub fn lookup_func(
        &self, 
        name: StrDescriptor
    ) -> Result<FuncSymbol, SymError> {
        self.func_defs.get(&name)
            .cloned()
            .ok_or(SymError::FuncNotFound(name))
    }
}

// Methods used in the name resolution pass
impl SymbolTable {
    pub fn nat_global_scope(&self) -> bool {
        self.common_ns.len() == 1
    }

    pub fn ndef_func(
        &mut self, 
        name: StrDescriptor, 
        type_: FuncType,
        linkage: Linkage,
    ) -> Result<(), SymError> {
        if !self.nat_global_scope() {
            return Err(SymError::FuncDefNotGlobal(name));
        }

        // currently, we ignore the linkage for function definitions
        if let Some(prev) = self.func_defs.get(&name) {
            if prev.is_definition {
                return Err(SymError::FuncRedefinition(name));
            }
            if prev.type_ != type_ {
                return Err(SymError::FuncRedefinition(name));
            }
        }

        let func_symbol = FuncSymbol {
            name,
            type_,
            linkage,
            is_definition: true,
        };
        self.func_defs.insert(name, func_symbol.clone());

        let cur_scope = self.common_ns.last_mut().unwrap();
        cur_scope.insert(name, CommonSymbol::Func(name));
        
        Ok(())
    }

    pub fn ndecl_func(
        &mut self, 
        name: StrDescriptor, 
        type_: FuncType,
        linkage: Linkage,
    ) -> Result<(), SymError> {
        if let Some(prev) = self.func_defs.get(&name) {
            if prev.type_ != type_ {
                return Err(SymError::FuncTypeMismatch(name));
            }
        } else {
            let func_symbol = FuncSymbol {
                name,
                type_,
                linkage,
                is_definition: false,
            };
            self.func_defs.insert(name, func_symbol);
        }

        // functions can be declared multiple times in the same scope,
        // as long as they have the same type.
        let cur_scope = self.common_ns.last_mut()
            .expect("Internal error: no current scope");
        if let Some(CommonSymbol::Var(_)) = cur_scope.get(&name) {
            // same function declarations can coexist,
            // but not with a variable of the same name
            return Err(SymError::DuplicateDecl(name));
        }
        cur_scope.insert(name, CommonSymbol::Func(name));

        Ok(())
    }
    
    pub fn nlookup_func(
        &self, 
        name: StrDescriptor
    ) -> Result<(), SymError> {
        for scope in self.common_ns.iter().rev() {
            if let Some(CommonSymbol::Func(_)) = scope.get(&name) {
                return Ok(());
            }
        }
        Err(SymError::FuncNotFound(name))
    }

    pub fn ndef_var(
        &mut self, 
        name: StrDescriptor, 
    ) -> Result<(), SymError> {
        let cur_scope = self.common_ns.last_mut()
            .expect("Internal error: no current scope");
        if cur_scope.contains_key(&name) {
            return Err(SymError::DuplicateDecl(name));
        }

        cur_scope.insert(name, CommonSymbol::Var(name));
        Ok(())
    }


    pub fn nlookup_var(
        &self, 
        name: StrDescriptor
    ) -> Result<(), SymError> {
        for scope in self.common_ns.iter().rev() {
            if let Some(CommonSymbol::Var(_)) = scope.get(&name) {
                return Ok(());
            }
        }
        Err(SymError::VarNotFound(name))
    }

    pub fn nlookup_var_cur(
        &self, 
        name: StrDescriptor
    ) -> Result<(), SymError> {
        if let Some(CommonSymbol::Var(_)) = self.common_ns.last()
            .and_then(|scope| scope.get(&name)) {
            Ok(())
        } else {
            Err(SymError::VarNotFound(name))
        }
    }
}
