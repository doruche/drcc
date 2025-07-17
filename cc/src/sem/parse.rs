use crate::common::*;
use crate::ast::{
    AstTopLevel,
    AstExpr,
};
use super::{
    TopLevel,
    Decl,
    BlockItem,
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
        }
    }

    pub fn parse(
        mut self, 
        ast: AstTopLevel,
    ) -> Result<TopLevel> {
        let strtb = ast.strtb;

        let mut nresolve_pass_decls = vec![];

        for decl in ast.decls {
            match self.nresolve_decl(decl) {
                Ok(decl) => nresolve_pass_decls.push(decl),
                Err((sym_e, span)) => Err(sym_e.to_error(&strtb, span))?,
            }
        }

        let mut lresolve_pass_decls = vec![];
        for decl in nresolve_pass_decls {
            lresolve_pass_decls.push(self.lresolve_decl(decl)?);
        }

        let decls = lresolve_pass_decls;

        Ok(TopLevel {
            decls,
            strtb,
            funcs: self.symtb.func_defs,
            static_vars: self.symtb.static_vars,
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