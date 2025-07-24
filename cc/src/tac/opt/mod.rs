//! Intermediate-level optimizations.

mod cfg;
mod constant_folding;
mod deadcode_elimination;

use std::collections::HashMap;

use super::{
    TopLevel,
    CodeGen,
    Opt,
    Function,
    FuncContext,
    LabelOperand,
    Insn,
    UnaryOp,
    BinaryOp,
    Operand,   
};

/// We only do intra-function optimizations.
impl CodeGen<Opt> {
    pub fn optimize_all(mut self, tac: TopLevel) -> TopLevel {
        let mut opted_funcs = HashMap::new();
        
        for (name, func) in tac.functions {
            let opted_func = self.opt_func(func);
            opted_funcs.insert(name, opted_func);
        }

        TopLevel {
            functions: opted_funcs,
            static_vars: tac.static_vars,
            strtb: tac.strtb,
        }

    }

    // currently a one-pass optimizer
    fn opt_func(&mut self, func: Function) -> Function {
        let post_constant_folding = self.constant_folding(func);

        post_constant_folding
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::fs::read_to_string;
    use std::io::Write;

    use crate::asm::CodeGen;
    use crate::lex::Lexer;
    use crate::ast::AstParser;
    use crate::sem::HirParser;
    use crate::tac::opt::cfg::Graph;
    use crate::tac::{Opt, TacCodeGen, TacTopLevel};
    use crate::lir::LirCodeGen;

    fn gen_tac(path: &str) -> (TacTopLevel, TacCodeGen<Opt>) {
        let input = read_to_string(path).unwrap();

        let mut lexer = Lexer::new(input);
        let (tokens, strtb) = lexer.lex().unwrap();

        let mut ast_parser = AstParser::new(tokens, strtb);
        let ast = ast_parser.parse_prog().unwrap();

        let mut hir_parser = HirParser::new();
        let hir = hir_parser.parse(ast).unwrap();

        let mut tac_codegen = TacCodeGen::new();
        tac_codegen.parse(hir)
    }

    fn test_inner(path: &str) {
        let input = read_to_string(path).unwrap();

        let mut lexer = Lexer::new(input);
        let (tokens, strtb) = lexer.lex().unwrap();

        let mut ast_parser = AstParser::new(tokens, strtb);
        let ast = ast_parser.parse_prog().unwrap();

        let mut hir_parser = HirParser::new();
        let hir = hir_parser.parse(ast).unwrap();

        let mut tac_codegen = TacCodeGen::new();
        let (tac, opt) = tac_codegen.parse(hir);
        let tac = opt.optimize_all(tac);

        let mut lir_parser = LirCodeGen::new();
        let (lir, lir_regalloc) = lir_parser.parse(tac);
        let (lir, lir_spill) = lir_regalloc.alloc(lir);
        let (lir, lir_canonic) = lir_spill.spill(lir);
        let lir = lir_canonic.canonic(lir);

        let mut asm_codegen = CodeGen::new();
        let (asm_top_level, _asm_opt) = asm_codegen.parse(lir);

        // println!("{}", asm_top_level.emit());

        // write to file
        let output_path = format!("{}.asm.opt.S", path);
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
    fn test_cfg() {
        let (mut tac, opt) = gen_tac("../testprogs/control_flow.c");

        println!("{}", tac.emit_code());

        let mut refactored_funcs = HashMap::new();        
        for (_, mut func) in tac.functions {
            let graph = Graph::build(func.body);
            let refactored_code = graph.emit();
            func.body = refactored_code;
            refactored_funcs.insert(func.name, func);
        }
        tac.functions = refactored_funcs;

        println!("{}", tac.emit_code());
    }

    fn test_dce(path: &str) {
        // write to file
        let output_path = format!("{}.dec.tac", path);
        let mut file = std::fs::File::create(output_path).unwrap();

        let (mut tac, mut opt) = gen_tac(path);

        file.write_fmt(format_args!("Unoptimized TAC:\n{}\n", tac.emit_code())).unwrap();

        let mut refactored_funcs = HashMap::new();        
        for (_, mut func) in tac.functions {
            let func = opt.deadcode_elimination(func);
            refactored_funcs.insert(func.name, func);
        }
        tac.functions = refactored_funcs;

        file.write_fmt(format_args!("Optimized TAC:\n{}\n", tac.emit_code())).unwrap();
    }

    #[test]
    fn test_dce_control_flow() {
        test_dce("../testprogs/control_flow.c");
    }

    #[test]
    fn test_dec_basic() {
        test_dce("../testprogs/basic.c");
    }

    #[test]
    fn test_dce_func() {
        test_dce("../testprogs/func.c");
    }
}