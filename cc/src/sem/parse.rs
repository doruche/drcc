use std::collections::HashMap;

use crate::common::*;
use crate::ast::{
    AstTopLevel,
    AstExpr,
};
use super::{
    TopLevel,
    LocalVarDecl,
    Function,
    BlockItem,
    StaticVar,
    Stmt,
    TypedExpr,
    Expr,
    UnaryOp,
    BinaryOp,
    SymbolTable,
    SymError,
    StaticVarSymbol,
    FuncSymbol,
};

#[derive(Debug)]
pub struct Parser {
    pub(super) symtb: SymbolTable,

    pub(super) label_counter: usize,
    pub(super) loop_labels: Vec<usize>,
    pub(super) local_var_id_counter: usize,

    pub(super) functions: HashMap<StrDescriptor, Function>,
    pub(super) static_vars: HashMap<StrDescriptor, StaticVar>,
}

impl Parser {
    pub(super) fn alloc_local_var(&mut self) -> usize {
        let id = self.local_var_id_counter;
        self.local_var_id_counter += 1;
        id
    }
}

impl Parser {
    pub fn new() -> Self {
        Self {
            symtb: SymbolTable::new(),
            label_counter: 0,
            loop_labels: vec![],
            local_var_id_counter: 0,
            functions: HashMap::new(),
            static_vars: HashMap::new(),
        }
    }

    pub fn parse(
        mut self, 
        ast: AstTopLevel,
    ) -> Result<TopLevel> {
        let strtb = ast.strtb;

        // name resolution
        for decl in ast.decls {
            match self.nresolve_decl(decl) {
                Ok(Some(_)) => panic!("Internal error: Top level parsing should not return a local variable declaration."),
                Ok(None) => {},
                Err((sym_e, span)) => Err(sym_e.to_error(&strtb, span))?,
            }
        }

        // label resolution
        let mut lresolver = super::lresolve::LResolver::new();
        for func in self.functions.values_mut() {
            lresolver.resolve_func(func)?;
        }

        Ok(TopLevel {
            strtb,
            funcs: self.functions,
            static_vars: self.static_vars,
        })
    }

    pub fn parse_expr(
        mut self,
        expr: AstExpr,
        strtb: StringPool,
    ) -> Result<TypedExpr> {
        match self.nresolve_expr(expr) {
            Ok(expr) => Ok(expr),
            Err((sym_e, span)) => Err(sym_e.to_error(&strtb, span))?,
        }
    }
}