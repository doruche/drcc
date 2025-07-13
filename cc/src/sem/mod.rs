//! Semantic analysis module.
//! AST -> HIR
//! Bake in type information, resolved names, etc.

mod hir;
mod symtb;
mod parse;

use symtb::{
    SymbolTable,
    Symbol,
    SymbolId,
    RawSymbol,
};