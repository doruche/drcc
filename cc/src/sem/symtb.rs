//! Symbol definition and symbol table management

use std::{collections::HashMap, fmt::Debug};

use crate::{common:: {
    DataType, Error, FuncType, Linkage, Span, StorageClass, StrDescriptor, StringPool
}, sem::hir::Param};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StaticVarSymbol {
    pub name: StrDescriptor,
    pub type_: DataType,
    pub linkage: Linkage,
    pub is_definition: bool, 
}
 
#[derive(Debug, Clone)]
pub struct FuncSymbol {
    pub name: StrDescriptor,
    pub type_: FuncType,
    pub linkage: Linkage,
    pub is_definition: bool,
}

/// In name resolution - we only need to know
/// the name of the symbol, not its type.
/// Whether types are matched will be determined in the type checking pass.
#[derive(Debug, Clone)]
pub enum CommonSymbol {
    Var(CommonVar),
    Func(StrDescriptor),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommonVar {
    pub name: StrDescriptor,
    pub local_id: Option<usize>,
    pub data_type: DataType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolType {
    Var,
    Func,
}

impl CommonSymbol {
    pub fn name(&self) -> StrDescriptor {
        match self {
            CommonSymbol::Var(var) => var.name,
            CommonSymbol::Func(name) => *name,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LabelSymbol {
    pub(super) name: StrDescriptor,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SymError {
    DuplicateDecl(StrDescriptor),
    VarNotFound(StrDescriptor),
    FuncNotFound(StrDescriptor),
    LabelNotFound(StrDescriptor),
    FuncRedefinition(StrDescriptor),
    StaticVarRedefinition(StrDescriptor),
    FuncDefNotGlobal(StrDescriptor),
    SymbolTypeMismatch {
        name: StrDescriptor,
        expected: SymbolType,
        found: SymbolType,
    },
    TypeMismatch {
        expected: DataType,
        found: DataType,
    },
    FuncTypeMismatch(StrDescriptor),
    LinkageMismatch(StrDescriptor),
    InvalidLValue,
    InvalidInitializer(StrDescriptor),
    InvalidArguments(StrDescriptor),
    Unimplemented(String),
    Other(String),
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
            SymError::StaticVarRedefinition(sd) => Error::semantic(
                format!("Static variable '{}' is already defined.", strtb.get(sd).unwrap()),
                span,
            ),
            SymError::FuncDefNotGlobal(sd) => Error::semantic(
                format!("Function '{}' is defined in a non-global scope.", strtb.get(sd).unwrap()),
                span,
            ),
            SymError::InvalidInitializer(name) => Error::semantic(
                format!("Invalid initializer for variable '{}'.", strtb.get(name).unwrap()),
                span,
            ),
            SymError::SymbolTypeMismatch { name, expected, found } => {
                let expected_str = match expected {
                    SymbolType::Var => "variable",
                    SymbolType::Func => "function",
                };
                let found_str = match found {
                    SymbolType::Var => "variable",
                    SymbolType::Func => "function",
                };
                Error::semantic(
                    format!("Symbol '{}' is a {} but was used as a {}.", 
                        strtb.get(name).unwrap(), found_str, expected_str),
                    span,
                )
            },
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
            SymError::InvalidArguments(name) => Error::semantic(
                format!("Invalid arguments for function '{}'.", strtb.get(name).unwrap()),
                span,
            ),
            SymError::LinkageMismatch(name) => Error::semantic(
                format!("Static linkage declaration follows non-static for symbol '{}'.", strtb.get(name).unwrap()),
                span,
            ),
            SymError::Unimplemented(msg) => Error::semantic(
                format!("Unimplemented feature: {}", msg),
                span,
            ),
            SymError::Other(msg) => Error::semantic(
                msg,
                span,
            ),
        }
    }
}


#[derive(Debug, Clone)]
pub struct SymbolTable {
    pub(super) common_ns: Vec<HashMap<StrDescriptor, CommonSymbol>>,
    pub(super) label_ns: HashMap<StrDescriptor, usize>,
    pub(super) func_defs: HashMap<StrDescriptor, FuncSymbol>,
    pub(super) static_vars: HashMap<StrDescriptor, StaticVarSymbol>,
}


// Common methods for the symbol table
impl SymbolTable {
    pub fn new() -> Self {
        Self {
            common_ns: vec![HashMap::new()],
            label_ns: HashMap::new(),
            func_defs: HashMap::new(),
            static_vars: HashMap::new(),
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
        storage_class: StorageClass,
    ) -> Result<(), SymError> {
        if !self.nat_global_scope() {
            return Err(SymError::FuncDefNotGlobal(name));
        }

        let mut linkage = match storage_class {
            StorageClass::Static => Linkage::Internal,
            StorageClass::Extern|StorageClass::Unspecified 
                => Linkage::External,
        };

        if let Some(prev) = self.func_defs.get(&name) {
            if prev.is_definition {
                return Err(SymError::FuncRedefinition(name));
            }
            if prev.type_ != type_ {
                return Err(SymError::FuncTypeMismatch(name));
            }
            linkage = match (prev.linkage, linkage) {
                (Linkage::Internal, _) => Linkage::Internal,
                (Linkage::External, Linkage::External) => prev.linkage,
                (Linkage::External, Linkage::Internal) => {
                    return Err(SymError::LinkageMismatch(name));
                },
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
        storage_class: StorageClass,
    ) -> Result<(), SymError> {
        if let Some(prev) = self.func_defs.get_mut(&name) {
            if prev.type_ != type_ {
                return Err(SymError::FuncTypeMismatch(name));
            }
            if let (Linkage::External, StorageClass::Static) = (prev.linkage, storage_class) {
                return Err(SymError::LinkageMismatch(name));
            }
        } else {
            let func_symbol = FuncSymbol {
                name,
                type_,
                linkage: match storage_class {
                    StorageClass::Static => Linkage::Internal,
                    StorageClass::Extern|StorageClass::Unspecified => Linkage::External,
                },
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
    
    pub fn ncheck_func(
        &self, 
        name: StrDescriptor
    ) -> Result<(), SymError> {
        for scope in self.common_ns.iter().rev() {
            match scope.get(&name) {
                Some(CommonSymbol::Func(_)) => return Ok(()),
                Some(CommonSymbol::Var(_)) => return Err(SymError::SymbolTypeMismatch { 
                    name, 
                    expected: SymbolType::Func, 
                    found: SymbolType::Var 
                }),
                None => continue,
            }
        }
        Err(SymError::FuncNotFound(name))
    }

    pub fn ndef_static_var(
        &mut self,
        // nresolve module should mangle names for in-block static variables
        // so that they do not conflict with global variables as well as each other.
        // this is not done here, but in the nresolve module.
        // for simplicity, only global variables and file-scope static variables
        // are supported for now.
        name: StrDescriptor,
        type_: DataType,
        storage_class: StorageClass,
        is_definition: bool,
    ) -> Result<(), SymError> {
        if !self.nat_global_scope() {
            return Err(SymError::Unimplemented(
                "Static variables in non-global scopes.".to_string(),
            ));
        }

        if let Some(prev) = self.static_vars.get_mut(&name) {
            if prev.is_definition && is_definition {
                return Err(SymError::StaticVarRedefinition(name));
            }
            if prev.type_ != type_ {
                return Err(SymError::TypeMismatch {
                    expected: prev.type_,
                    found: type_,
                });
            }

            prev.is_definition = is_definition;
            match (prev.linkage, storage_class) {
                (Linkage::Internal, StorageClass::Unspecified) =>
                    return Err(SymError::LinkageMismatch(name)),
                (Linkage::External, StorageClass::Static) => 
                    return Err(SymError::LinkageMismatch(name)),
                _ => {},
            }
        } else {
            let static_var_symbol = StaticVarSymbol {
                name,
                type_,
                linkage: match storage_class {
                    StorageClass::Static => Linkage::Internal,
                    StorageClass::Extern | StorageClass::Unspecified => Linkage::External,
                },
                is_definition,
            };
            self.static_vars.insert(name, static_var_symbol);
            self.common_ns.last_mut()
                .expect("Internal error: no current scope")
                .insert(name, CommonSymbol::Var(CommonVar { name, data_type: type_, local_id: None }));
        }

        Ok(())
    }

    pub fn ndef_var(
        &mut self, 
        name: StrDescriptor,
        data_type: DataType,
        local_id: Option<usize>,
    ) -> Result<(), SymError> {
        let cur_scope = self.common_ns.last_mut()
            .expect("Internal error: no current scope");
        if cur_scope.contains_key(&name) {
            return Err(SymError::DuplicateDecl(name));
        }

        cur_scope.insert(name, CommonSymbol::Var(CommonVar { name, data_type, local_id }));
        Ok(())
    }

    pub fn nlookup_var(
        &self, 
        name: StrDescriptor
    ) -> Result<CommonVar, SymError> {
        for scope in self.common_ns.iter().rev() {
            match scope.get(&name) {
                Some(CommonSymbol::Var(var)) => return Ok(*var),
                Some(CommonSymbol::Func(_)) => return Err(SymError::SymbolTypeMismatch { 
                    name, 
                    expected: SymbolType::Var, 
                    found: SymbolType::Func 
                }),
                None => continue,
            }
        }
        Err(SymError::VarNotFound(name))
    }

    pub fn ncheck_var_cur(
        &self, 
        name: StrDescriptor
    ) -> Result<(), SymError> {
        match self.common_ns.last().and_then(|scope| scope.get(&name)) {
            Some(CommonSymbol::Var(_)) => Ok(()),
            Some(CommonSymbol::Func(_)) => Err(SymError::SymbolTypeMismatch { 
                name, 
                expected: SymbolType::Var, 
                found: SymbolType::Func 
            }),
            None => Err(SymError::VarNotFound(name)),
        }
    }
}
