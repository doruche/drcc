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

#[cfg(test)]
mod tests {
    use super::*;

    fn test_inner(path: &str) {
        let input = std::fs::read_to_string(path).unwrap();
        let mut lexer = crate::lex::Lexer::new(input.into());
        let (tokens, strtb) = lexer.lex().unwrap();
        
        let mut parser = crate::ast::AstParser::new(tokens, strtb);
        let ast = parser.parse_prog().unwrap();

        let mut sem_parser = Parser::new();
        let result = sem_parser.parse(ast);

        match result {
            Ok(hir) => println!("{:#?}", hir),
            Err(e) => println!("{}", e),
        }        
    }

    #[test]
    fn test_basic() {
        test_inner("../testprogs/return_42.c");
    }

    #[test]
    fn test_var() {
        test_inner("../testprogs/var.c");
    }
}