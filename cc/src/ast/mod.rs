//! Abstract Syntax Tree (AST) module.
//! Tokens -> AST

use std::fmt::Display;

mod parser;
mod expr;
mod stmt;
mod decl;
mod ast;

use parser::Parser;
use ast::{
    TopLevel,
    Decl,
    Expr,
    Stmt,
    BlockItem,
    UnaryOp,
    BinaryOp,
    ForInit,
    Param,
};
use crate::common::{DataType, Span, StrDescriptor};

pub use parser::Parser as AstParser;
pub use ast::{
    TopLevel as AstTopLevel,
    Decl as AstDecl,
    Expr as AstExpr,
    Stmt as AstStmt,
    Param as AstParam,
    BlockItem as AstBlockItem,
    UnaryOp as AstUnaryOp,
    BinaryOp as AstBinaryOp,
    ForInit as AstForInit,
};

#[cfg(test)]
mod tests {
    use std::fs::read_to_string;

    use crate::lex::Lexer;

    use super::*;

    #[test]
    fn test_expr() {
        let input = "a = 5 = 7 + 2";
        let mut lexer = Lexer::new(input.into());
        let (tokens, pool) = lexer.lex().unwrap();
        let mut parser = Parser::new(tokens, pool);
        match parser.parse_expr() {
            Ok(expr) => {
                println!("{:#?}", expr);
            }
            Err(e) => {
                eprintln!("{}", e);
            }
        }
    }

    fn test_inner(path: &str) {
        let input = read_to_string(path).unwrap();
        let mut lexer = Lexer::new(input.into());
        let (tokens, strtb) = lexer.lex().unwrap();

        let mut parser = Parser::new(tokens, strtb);
        match parser.parse_prog() {
            Ok(prog) => {
                println!("{:#?}", prog);
            }
            Err(e) => {
                eprintln!("{}", e);
            }
        }

    }

    #[test]
    fn test_basic() {
        test_inner("../testprogs/basic.c");
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
    fn test_func() {
        test_inner("../testprogs/func.c");
    }

    #[test]
    fn test_static() {
        test_inner("../testprogs/static.c");
    }
}