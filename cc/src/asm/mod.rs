//! Assembly code emission module
//! LIR -> RISC-V Assembly

mod riscv;
mod codegen;
mod emit;

use std::marker::PhantomData;
use crate::common::*;

use riscv::{
    TopLevel,
    Insn,
    Function,
    StaticVar,
    BssSegment,
    DataSegment,
    LabelOperand,
};

pub use riscv::{
    Register,
    TopLevel as AsmTopLevel,
    Function as AsmFunction,
    StaticVar as AsmStaticVar,
    DataSegment as AsmDataSegment,
    BssSegment as AsmBssSegment,
    Insn as AsmInsn,
    LabelOperand as AsmLabelOperand,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Parse;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Opt;

#[derive(Debug)]
pub struct FuncContext {
    pub name: StrDescriptor,
    pub callee_saved: Vec<(Register, isize)>,
    pub frame_size: usize,
}


#[derive(Debug)]
pub struct CodeGen<Stage = Parse> {
    pub cur_cx: Option<FuncContext>,

    pub _stage: PhantomData<Stage>,
}

#[cfg(test)]
mod tests {
    use std::fs::read_to_string;

    use crate::lex::Lexer;
    use crate::ast::AstParser;
    use crate::sem::HirParser;
    use crate::tac::TacCodeGen;
    use crate::lir::LirCodeGen;

    use super::*;

    fn test_inner(path: &str) {
        let input = read_to_string(path).unwrap();

        let mut lexer = Lexer::new(input);
        let (tokens, strtb) = lexer.lex().unwrap();

        let mut ast_parser = AstParser::new(tokens, strtb);
        let ast = ast_parser.parse_prog().unwrap();

        let mut hir_parser = HirParser::new();
        let hir = hir_parser.parse(ast).unwrap();

        let mut tac_codegen = TacCodeGen::new();
        let (tac, _opt) = tac_codegen.parse(hir);

        let mut lir_parser = LirCodeGen::new();
        let (lir, lir_regalloc) = lir_parser.parse(tac);
        let (lir, lir_spill) = lir_regalloc.alloc(lir);
        let (lir, lir_canonic) = lir_spill.spill(lir);
        let lir = lir_canonic.canonic(lir);

        let mut asm_codegen = CodeGen::new();
        let (asm_top_level, _asm_opt) = asm_codegen.parse(lir);

        // println!("{}", asm_top_level.emit());

        // write to file
        let output_path = format!("{}.asm.S", path);
        std::fs::write(output_path, asm_top_level.emit()).unwrap();
    }

    #[test]
    fn test_basic() {
        test_inner("../testprogs/basic.c");
    }

    #[test]
    fn test_func() {
        test_inner("../testprogs/func.c");
    }

    #[test]
    fn test_control_flow() {
        test_inner("../testprogs/control_flow.c");
    }
}