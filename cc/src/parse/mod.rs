//! Parsing module.

use std::fmt::Display;

mod parser;
mod expr;
mod stmt;

pub use parser::Parser;
pub use expr::Expr;
pub use stmt::Stmt;

#[cfg(test)]
mod tests {
    use std::fs::read_to_string;

    use crate::lex::Lexer;

    use super::*;

    #[test]
    fn test_basic() {
        let input = read_to_string("../testprogs/return_2.c").unwrap();
        let mut lexer = Lexer::new(input);
        let tokens = lexer.lex().unwrap();
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