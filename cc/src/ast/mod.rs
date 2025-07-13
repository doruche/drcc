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
};
use crate::common::{DataType, Span, StrDescriptor};

pub use parser::Parser as AstParser;
pub use ast::{
    TopLevel as AstTopLevel,
    Decl as AstDecl,
    Expr as AstExpr,
    Stmt as AstStmt,
    BlockItem as AstBlockItem,
    UnaryOp as AstUnaryOp,
    BinaryOp as AstBinaryOp,
};

#[cfg(test)]
mod tests {
    use std::fs::read_to_string;

    use crate::lex::Lexer;

    use super::*;

    #[test]
    fn test_basic() {
        let input = read_to_string("../testprogs/return_2.c").unwrap();
        let mut lexer = Lexer::new(input);
        let (tokens, pool) = lexer.lex().unwrap();
        let mut parser = Parser::new(tokens, pool);
        match parser.parse_prog() {
            Ok(stmt) => {
                println!("{:#?}", stmt);
            }
            Err(e) => {
                eprintln!("{}", e);
            }
        }
    }

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

    #[test]
    fn test_binary() {
        let input = "int main(void) { return 1 <= 3 + 4 * 2 && 3 > 1; }";
        let mut lexer = Lexer::new(input.into());
        let (tokens, pool) = lexer.lex().unwrap();
        // for token in &tokens {
        //     println!("{}", token);
        // }
        let mut parser = Parser::new(tokens, pool);
        match parser.parse_prog() {
            Ok(stmt) => {
                println!("{:#?}", stmt);
            }
            Err(e) => {
                eprintln!("{}", e);
            }
        }
    }
}