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
    use std::collections::HashMap;
    use std::fs::read_to_string;
    use std::io::Write;

    use crate::lex::Lexer;
    use crate::ast::AstParser;
    use crate::sem::HirParser;
    use crate::tac::{TacCodeGen, TacTopLevel};
    use crate::lir::LirCodeGen;

    use super::*;

    fn tac2asm(tac: TacTopLevel) -> AsmTopLevel {
        let mut lir_codegen = LirCodeGen::new();
        let (lir, lir_regalloc) = lir_codegen.parse(tac);
        let (lir, lir_spill) = lir_regalloc.alloc(lir);
        let (lir, lir_canonic) = lir_spill.spill(lir);
        let lir = lir_canonic.canonic(lir);

        let mut asm_codegen = CodeGen::new();
        let (asm, _opt) = asm_codegen.parse(lir);
        asm
    }

    fn test_inner(
        path: &str,
        constant_folding: bool,
        deadcode_elimination: bool,
        copy_propagation: bool,
        opt_time: usize,
    ) {
        let input = read_to_string(path).unwrap();
        let mut output_path = String::from(path);
        if constant_folding {
            output_path.push_str(".cf");
        }
        if deadcode_elimination {
            output_path.push_str(".dce");
        }
        if copy_propagation {
            output_path.push_str(".cp");
        }
        let output_path = format!("{}.{}.S", output_path, opt_time);
        let mut file = std::fs::File::create(output_path).unwrap();

        let mut lexer = Lexer::new(input);
        let (tokens, strtb) = lexer.lex().unwrap();

        let mut ast_parser = AstParser::new(tokens, strtb);
        let ast = ast_parser.parse_prog().unwrap();

        let mut hir_parser = HirParser::new();
        let hir = hir_parser.parse(ast).unwrap();

        let mut tac_codegen = TacCodeGen::new();
        let (tac, mut opt) = tac_codegen.parse(hir);
        let mut tac_to_opt = tac.clone();

        let mut refactored_funcs = HashMap::new();
        for (_, mut func) in tac_to_opt.functions {
            for i in 0..opt_time {
                if constant_folding {
                    func = opt.constant_folding(func);
                }
                if deadcode_elimination {
                    func = opt.deadcode_elimination(func);
                }
                if copy_propagation {
                    func = opt.copy_propagation(func);
                }
            }
            refactored_funcs.insert(func.name, func);
        }
        tac_to_opt.functions = refactored_funcs;

        let asm = tac2asm(tac_to_opt);
        let asm_str = asm.emit();
        file.write_all(asm_str.as_bytes()).unwrap();
    }

    #[test]
    fn test_basic_opt() {
        test_inner(
            "../testprogs/basic.c",
            true,
            true,
            true,
            2,
        );
    }

    #[test]
    fn test_basic() {
        test_inner(
            "../testprogs/basic.c",
            false,
            false,
            false,
            0,
        );
    }

    #[test]
    fn test_control_flow_opt() {
        test_inner(
            "../testprogs/control_flow.c",
            true,
            true,
            true,
            2,
        );
    }

    #[test]
    fn test_control_flow() {
        test_inner(
            "../testprogs/control_flow.c",
            false,
            false,
            false,
            0,
        );
    }

    #[test]
    fn test_func_opt() {
        test_inner(
            "../testprogs/func.c",
            true,
            true,
            true,
            2,
        );
    }

    #[test]
    fn test_func() {
        test_inner(
            "../testprogs/func.c",
            false,
            false,
            false,
            0,
        );
    }
}