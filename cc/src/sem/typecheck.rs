//! Final pass of semantic analysis.
//! Transforms the HirTopLevel into a fully typed and resolved structure.
use crate::common::*;
use super::{
    Parser,
    SymbolTable,
    SymError,
    StaticVarSymbol,
    FuncSymbol,
    TopLevel,
    ForInit,
    Param,
    LocalVarDecl,
    BlockItem,
    Stmt,
    TypedExpr,
    Expr,
    UnaryOp,
    BinaryOp,
};

pub struct TypeChecker {

}

impl TypeChecker {

}