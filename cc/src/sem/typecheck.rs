//! Final pass of semantic analysis.
//! Transforms the HirTopLevel into a fully typed and resolved structure.
use crate::common::*;
use super::{
    Parser,
    SymbolTable,
    SymError,
    VarSymbol,
    FuncSymbol,
    TopLevel,
    Decl,
    BlockItem,
    Stmt,
    TypedExpr,
    Expr,
    UnaryOp,
    BinaryOp,
};

impl Parser {
    pub(super) fn type_decl(
        &mut self,
        decl: Decl,
    ) -> Result<Decl> {
        todo!()
    }

    pub(super) fn type_block_item(
        &mut self,
        item: BlockItem,
    ) -> Result<BlockItem> {
        todo!()
    }

    pub(super) fn type_stmt(
        &mut self,
        stmt: Stmt,
    ) -> Result<Stmt> {
        todo!()
    }

    pub(super) fn type_expr(
        &mut self,
        expr: Expr,
    ) -> Result<TypedExpr> {
        todo!()
    }
}