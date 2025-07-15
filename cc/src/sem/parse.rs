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

    pub(super) label_counter: usize,
    pub(super) loop_labels: Vec<usize>,
}

impl Parser {
    pub fn new() -> Self {
        Self {
            symtb: SymbolTable::new(),
            label_counter: 0,
            loop_labels: vec![],
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

        let mut lresolve_pass_decls = vec![];
        for decl in nresolve_pass_decls {
            lresolve_pass_decls.push(self.lresolve_decl(decl)?);
        }

        let decls = lresolve_pass_decls;

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