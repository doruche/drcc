//! Semantic analysis module.
//! AST -> HIR
//! Bake in type information, resolved names, etc.

mod hir;
mod symtb;
mod parse;
mod nresolve;
mod typecheck;

use symtb::{
    SymbolTable,
    VarSymbol,
    FuncSymbol,
    SymError,
};
use hir::{
    TopLevel,
    Decl,
    BlockItem,
    Stmt,
    TypedExpr,
    Expr,
    UnaryOp,
    BinaryOp,
};
use parse::Parser;

pub use hir::{
    TopLevel as HirTopLevel,
    Decl as HirDecl,
    BlockItem as HirBlockItem,
    Stmt as HirStmt,
    TypedExpr as HirTypedExpr,
    Expr as HirExpr,
    UnaryOp as HirUnaryOp,
    BinaryOp as HirBinaryOp,
};
pub use parse::Parser as HirParser;