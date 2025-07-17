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
    StaticVar as TacStaticVar,
    TopLevel as TacTopLevel,
    UnaryOp as TacUnaryOp,
    BinaryOp as TacBinaryOp,
    LabelOperand as TacLabelOperand,
    AutoGenLabel as TacAutoGenLabel,
};

use tac::{
    Operand,
    Insn,
    Function,
    StaticVar,
    TopLevel,
    UnaryOp,
    BinaryOp,
    LabelOperand,
    AutoGenLabel,
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

    fn test_inner(path: &str) {
        let input = read_to_string(path).unwrap();
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
                println!("{:#}", tac.emit_code());
                println!("{:#}", tac.emit_static_vars());
            }
            Err(e) => {
                eprintln!("Error: {}", e);
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