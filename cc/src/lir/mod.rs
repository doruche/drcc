//! Low-Level Intermediate Representation (LIR) module
//! TAC -> LIR
//! In this module, we will:
//! 1. Convert the three-address code (TAC) into a low-level intermediate representation (LIR).
//! 2. Instruction canonicalization.
//! 3. Register allocation.

mod lir;
mod blocks;
mod codegen;
mod emit;

use lir::{
    Operand,
    Insn,
    Function,
    LabelOperand,
    LabelSignature,
    StaticVar,
    TopLevel,
    DataSegment,
    BssSegment,
    IntermediateInsn,
};

pub use lir::{
    Operand as LirOperand,
    Insn as LirInsn,
    Function as LirFunction,
    StaticVar as LirStaticVar,
    TopLevel as LirTopLevel,
    DataSegment as LirDataSegment,
    BssSegment as LirBssSegment,
};

#[cfg(test)]
mod tests {
    use std::fs::read_to_string;

    use crate::lex::Lexer;
    use crate::ast::AstParser;
    use crate::sem::HirParser;
    use crate::tac::TacCodeGen;
    use crate::lir::codegen::CodeGen;

    use super::*;

    fn test_inner(path: &str) {
        let input = read_to_string(path).unwrap();

        let mut lexer = Lexer::new(input);
        let (tokens, strtb) = lexer.lex().unwrap();

        let mut parser = AstParser::new(tokens, strtb);
        let ast = parser.parse_prog().unwrap();

        let mut parser = HirParser::new();
        let hir = parser.parse(ast).unwrap();

        let mut parser = TacCodeGen::new();
        let tac = parser.parse(hir).unwrap();

        let mut codegen_parse = CodeGen::new();
        let (lir, codegen_canonic) = codegen_parse.parse(tac);

        println!("{}", lir.emit_code());
    }

    #[test]
    fn test_basic() {
        test_inner("../testprogs/basic.c");
    }

    #[test]
    fn test_control_flow() {
        test_inner("../testprogs/control_flow.c");
    }

    #[test]
    fn test_func() {
        test_inner("../testprogs/func.c");
    }
}