//! Semantic analysis module.
//! AST -> HIR
//! Bake in type information, resolved names, etc.
//! Current passes:
//! 1. Name resolution
//! 2. Label resolution: attach according labels to break/continue statements
//! 3. Type checking (not implemented yet)



mod hir;
mod symtb;
mod parse;
mod nresolve;
mod lresolve;
mod typecheck;

use symtb::{
    SymbolTable,
    SymError,
};
use hir::{
    TopLevel,
    Decl,
    BlockItem,
    Stmt,
    ForInit,
    Param,
    TypedExpr,
    Expr,
    UnaryOp,
    BinaryOp,
    Variable,
};
use parse::Parser;

pub use hir::{
    TopLevel as HirTopLevel,
    Decl as HirDecl,
    BlockItem as HirBlockItem,
    Stmt as HirStmt,
    ForInit as HirForInit,
    Param as HirParam,
    Variable as HirVariable,
    TypedExpr as HirTypedExpr,
    Expr as HirExpr,
    UnaryOp as HirUnaryOp,
    BinaryOp as HirBinaryOp,
};
pub use parse::Parser as HirParser;
pub use symtb::{
    FuncSymbol,
    StaticVarSymbol,
    CommonVar,
};

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
            Ok(hir) => {
                println!("{:#?}", hir.decls);
                println!("String Table\n{:#?}", hir.strtb);
                println!("Function Definitions\n{}", hir.dump_funcs());
                println!("Static Variables\n{}", hir.dump_static_vars());
            },
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

    #[test]
    fn test_if() {
        test_inner("../testprogs/if.c");
    }

    #[test]
    fn test_ternary() {
        test_inner("../testprogs/ternary.c");
    }

    #[test]
    fn test_compound() {
        test_inner("../testprogs/compound.c");
    }

    #[test]
    fn test_loop() {
        test_inner("../testprogs/loop.c");
    }

    #[test]
    fn test_control_flow() {
        test_inner("../testprogs/control_flow.c");
    }

    #[test]
    fn test_func() {
        test_inner("../testprogs/func.c");
    }

    #[test]
    fn test_static() {
        test_inner("../testprogs/static.c");
    }
}