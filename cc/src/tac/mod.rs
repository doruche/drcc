//! Three-Address Code (TAC/MIR) module.
//! HIR -> TAC

use std::{collections::{HashMap, HashSet}, fmt::Display, marker::PhantomData};

mod codegen;
mod opt;
mod emit;
mod tac;

pub use tac::{
    Operand as TacOperand,
    Insn as TacInsn,
    Function as TacFunction,
    StaticVar as TacStaticVar,
    TopLevel as TacTopLevel,
    UnaryOp as TacUnaryOp,
    BinaryOp as TacBinaryOp,
    Param as TacParam,
    LocalVar as TacLocalVar,
    LabelOperand as TacLabelOperand,
    AutoGenLabel as TacAutoGenLabel,
};
pub use CodeGen as TacCodeGen;

use crate::common::*;
use tac::{
    Operand,
    Insn,
    LocalVar,
    Function,
    StaticVar,
    TopLevel,
    UnaryOp,
    BinaryOp,
    Param,
    LabelOperand,
    AutoGenLabel,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parse;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Opt;

#[derive(Debug, Clone)]
pub struct FuncContext {
    pub local_vars: HashMap<usize, LocalVar>,
}

#[derive(Debug)]
pub struct CodeGen<Stage = Parse> {
    pub cur_cx: Option<FuncContext>,

    pub static_vars: HashSet<(StrDescriptor, DataType)>,
    _stage: PhantomData<Stage>,
}

impl CodeGen {
    pub fn new() -> Self {
        Self {
            cur_cx: None,
            static_vars: HashSet::new(),
            _stage: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs::read_to_string;


    use crate::lex::Lexer;
    use crate::ast::AstParser;
    use crate::sem::HirParser;

    use super::*;

    fn test_inner(path: &str) {
        let input = read_to_string(path).unwrap();
        let mut lexer = Lexer::new(input.into());
        let (tokens, strtb) = lexer.lex().unwrap();

        let mut parser = AstParser::new(tokens, strtb);
        let prog = parser.parse_prog().unwrap();
        
        let mut parser = HirParser::new();
        let prog = parser.parse(prog).unwrap();

        let mut parser = CodeGen::new();
        let (tac, optimizer) = parser.parse(prog);
        
        println!("{:#}", tac.emit_code());
        println!("{:#}", tac.emit_static_vars());
    }

    fn test_inner_with_opt(path: &str) {
        let input = read_to_string(path).unwrap();
        let mut lexer = Lexer::new(input.into());
        let (tokens, strtb) = lexer.lex().unwrap();

        let mut parser = AstParser::new(tokens, strtb);
        let prog = parser.parse_prog().unwrap();
        
        let mut parser = HirParser::new();
        let prog = parser.parse(prog).unwrap();

        let mut codegen = CodeGen::new();
        let (tac, optimizer) = codegen.parse(prog);
        let opted_tac = optimizer.optimize_all(tac);

        println!("{:#}", opted_tac.emit_code());
        println!("{:#}", opted_tac.emit_static_vars());
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
    fn test_basic_with_opt() {
        test_inner_with_opt("../testprogs/basic.c");
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

    #[test]
    fn test_long() {
        test_inner("../testprogs/long.c");
    }

    #[test]
    fn test_cast() {
        test_inner("../testprogs/cast.c");
    }

    #[test]
    fn test_reg() {
        test_inner("../testprogs/reg.c");
    }

    #[test]
    fn test_reg_with_opt() {
        test_inner_with_opt("../testprogs/reg.c");
    }
}