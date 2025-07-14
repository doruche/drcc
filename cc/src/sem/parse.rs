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
    VarSymbol,
    FuncSymbol,
};

#[derive(Debug)]
pub struct Parser {
    pub(super) symtb: SymbolTable,
}

impl Parser {
    pub fn new() -> Self {
        Self {
            symtb: SymbolTable::new(),
        }
    }

    pub fn parse(
        mut self, 
        ast: AstTopLevel,
    ) -> Result<TopLevel> {
        // we'll do several passes here.
        // 1. build the symbol table, while resolving names
        // 2. type check
        // currently, we only do the first pass.

        let mut decls = vec![];
        let strtb = ast.strtb;

        for decl in ast.decls {
            match self.nresolve_decl(decl) {
                Ok(decl) => decls.push(decl),
                Err((sym_e, span)) => match sym_e {
                    SymError::DuplicateDecl(sd) =>
                        return Err(Error::semantic(format!(
                            "Symbol '{}' is already defined in this scope.",
                            strtb.get(sd).unwrap()
                        ), span)),
                    SymError::NotFound(sd) =>
                        return Err(Error::semantic(format!("Symbol '{}' is not defined.",
                        strtb.get(sd).unwrap()), span)),
                    SymError::InvalidLValue =>
                        return Err(Error::semantic(format!(
                            "Left-hand side of assignment must be a variable.",
                        ), span)),
                }
            }
        }

        Ok(TopLevel {
            decls,
            strtb,
        })
    }

    pub fn parse_expr(
        mut self,
        expr: AstExpr,
        strtb: StringPool,
    ) -> Result<TypedExpr> {
        match self.nresolve_expr(expr) {
            Ok(expr) => Ok(expr),
            Err((sym_e, span)) => match sym_e {
                SymError::DuplicateDecl(sd) =>
                    Err(Error::Semantic(format!(
                        "Ln {} Col{}:{}\tSymbol '{}' is already defined in this scope.",
                        span.line, span.column, span.end_col(), strtb.get(sd).unwrap(),
                    ))),
                SymError::NotFound(sd) =>
                    Err(Error::Semantic(format!(
                        "Ln {} Col{}:{}\tSymbol '{}' not found.",
                        span.line, span.column, span.end_col(), strtb.get(sd).unwrap(),
                    ))),
                SymError::InvalidLValue =>
                    Err(Error::Semantic(format!(
                        "Ln {} Col{}:{}\tLeft-hand side of assignment must be a variable.",
                        span.line, span.column, span.end_col(),
                    ))),
            }
        }
    }
}