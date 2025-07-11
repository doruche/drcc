//! Lexer module for tokenizing source code.

mod lexer;

use std::fmt::Display;

pub use lexer::Lexer;

// #[derive(Debug)]
// pub enum Error {
//     
// }

#[derive(Debug)]
pub struct Error {
    msg: String,
}

impl Error {
    pub fn new(msg: &str) -> Self {
        Error { msg: msg.to_string() }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}


#[cfg(test)]
mod tests {
    use std::fs::read_to_string;

    use super::*;

    #[test]
    fn test_basic() {
        let input = read_to_string("../testprogs/return_2.c").unwrap();
        let lexer = Lexer::new(input);
        match lexer.lex() {
            Ok(tokens) => {
                for token in tokens {
                    println!("{:?}", token);
                }
            }
            Err(e) => {
                eprintln!("{}", e);
            }
        }
    }
}