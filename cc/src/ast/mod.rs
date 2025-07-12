//! Abstract Syntax Tree (AST) module.
//! Tokens -> AST

use std::fmt::Display;

mod parser;
mod expr;
mod stmt;

pub use parser::Parser as AstParser;
pub(super) use parser::Parser;
pub use expr::Expr;
pub use stmt::Stmt;

use crate::common::{DataType, Span, StrDescriptor};

#[derive(Debug, Clone)]
pub struct TopLevel {
    pub decls: Vec<TopDeclaration>,
}

#[derive(Debug, Clone)]
pub enum TopDeclaration {
    FuncDecl {
        return_type: (DataType, Span),
        name: (StrDescriptor, Span),
        // params,
        body: Vec<Stmt>,
    },
    GloblVar {
        name: (StrDescriptor, Span),
        data_type: DataType,
        // initial value,
    }
}

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
        let mut parser = Parser::new(tokens);
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
    fn test_binary() {
        let input = "int main(void) { return 1 * 2 - 3 * (4 + 5); }";
        let mut lexer = Lexer::new(input.into());
        let (tokens, pool) = lexer.lex().unwrap();
        let mut parser = Parser::new(tokens);
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