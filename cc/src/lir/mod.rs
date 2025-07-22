//! Low-Level Intermediate Representation (LIR) module
//! TAC -> LIR
//! This stage includes following passes:
//! 1. Convert the TAC code into an incomplete LIR.
//! 2. Register allocation.
//! 3. Instruction canonicalization.
//! 4. Prologue/epilogue insertion.
//! NOTE Since we start register allocation immediately after the LIR conversion, 
//! we have to ensure not generating illegal instructions such as:
//! 1. mv   a(mem1), b(mem2),
//! 2. addi a(mem1), b(mem2), 0.
//! We must break these instructions into simpler ones that only use virtual registers or physical registers.
//! Otherwise, the register allocator will not be able to handle them correctly.

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
        let (tac, optimizer) = parser.parse(hir);

        let mut codegen_parse = CodeGen::new();
        let (lir, codegen_canonic) = codegen_parse.parse(tac);

        println!("{}", lir.emit());
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

    #[test]
    fn test_static() {
        test_inner("../testprogs/static.c");
    }

    #[test]
    fn test_cast() {
        test_inner("../testprogs/cast.c");
    }

    #[test]
    fn test_reg() {
        test_inner("../testprogs/reg.c");
    }
}