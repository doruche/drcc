//! Three-Address Code (TAC/MIR) module.
//! HIR -> TAC

use std::fmt::Display;

mod codegen;
mod emit;
mod tac;

pub use codegen::Parser as TacParser;
pub use tac::{
    Operand as TacOperand,
    Insn as TacInsn,
    Function as TacFunction,
    TopLevel as TacTopLevel,
    UnaryOp as TacUnaryOp,
    BinaryOp as TacBinaryOp,
};

use tac::{
    Operand,
    Insn,
    Function,
    TopLevel,
    UnaryOp,
    BinaryOp,
};


#[cfg(test)]
mod tests {
    use std::fs::read_to_string;


    use crate::lex::Lexer;
    use crate::ast::AstParser;
    use crate::sem::HirParser;

    use super::*;

    #[test]
    fn test_expr() {
        let mut lexer = Lexer::new("1<=3*7>5==1".into());
        let (tokens, pool) = lexer.lex().unwrap();

        let mut parser = AstParser::new(tokens, pool);
        let expr = parser.parse_expr().unwrap();
        let strtb = parser.strtb();

        let mut parser = HirParser::new();
        let expr = parser.parse_expr(expr, strtb).unwrap();

        let result = codegen::parse_expr(expr, &mut 0, &mut 0);
        match result {
            Ok((operand, insns)) => {
                println!("Operand: {:?}", operand);
                if let Some(insns) = insns {
                    for insn in insns {
                        println!("Insn: {:?}", insn);
                    }
                }
            }
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
    }

    #[test]
    fn test_prog() {
        // let input = "int main(void) { return 1 * 2 - 3 * (4 + 5); }";
        let input = read_to_string("../testprogs/basic.c").unwrap();
        let mut lexer = Lexer::new(input.into());
        let (tokens, strtb) = lexer.lex().unwrap();

        let mut parser = AstParser::new(tokens, strtb);
        let prog = parser.parse_prog().unwrap();
        
        let mut parser = HirParser::new();
        let prog = parser.parse(prog).unwrap();

        let mut parser = TacParser::new(prog);
        let result = parser.parse();
        match result {
            Ok(tac) => {
                println!("{:#}", tac.emit());
            }
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
    }
}