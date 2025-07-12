//! Lexer module for tokenizing source code.
//! Text -> Tokens

mod lexer;

use std::fmt::Display;

pub use lexer::Lexer;


#[cfg(test)]
mod tests {
    use std::fs::read_to_string;

    use super::*;

    #[test]
    fn test_basic() {
        let input = read_to_string("../testprogs/return_2.c").unwrap();
        let lexer = Lexer::new(input);
        match lexer.lex() {
            Ok((tokens, pool)) => {
                for token in tokens {
                    println!("{:?}", token);
                }
                println!("String Pool: {:?}", pool);
            },
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
    }
}