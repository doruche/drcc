//! Intermediate-level optimizations.

mod cfg;
mod constant_folding;
mod deadcode_elimination;
mod copy_propagation;
mod deadstore_elimination;

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
        let post_deadcode_elimination = self.deadcode_elimination(post_constant_folding);
        let post_copy_propagation = self.copy_propagation(post_deadcode_elimination);

        post_copy_propagation
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
    use crate::tac::opt::deadstore_elimination;
    use crate::tac::{Opt, TacCodeGen, TacFunction, TacTopLevel};
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

    fn test_opt(
        path: &str,
        constant_folding: bool,
        deadcode_elimination: bool,
        copy_propagation: bool,
        deadstore_elimination: bool,
        opt_time: usize,
    ) -> TacTopLevel {
        // write to file
        let mut output_path = String::new();
        output_path.push_str(path);
        if constant_folding {
            output_path.push_str(".cf");
        }
        if deadcode_elimination {
            output_path.push_str(".dce");
        }
        if copy_propagation {
            output_path.push_str(".cp");
        }
        if deadstore_elimination {
            output_path.push_str(".dse");
        }
        output_path.push_str(&format!(".{}.tac", opt_time));
        
        let mut file = std::fs::File::create(output_path).unwrap();

        let (mut tac, mut opt) = gen_tac(path);

        let mut refactored_funcs = HashMap::new();
        for (_, mut func) in tac.functions {
            for i in 0..opt_time {
                if constant_folding {
                    func = opt.constant_folding(func);
                }
                if copy_propagation {
                    func = opt.copy_propagation(func);
                }
                if deadcode_elimination {
                    func = opt.deadcode_elimination(func);
                }
                if deadstore_elimination {
                    func = opt.deadstore_elimination(func);
                }
            }
            refactored_funcs.insert(func.name(), func);
        }
        tac.functions = refactored_funcs;

        file.write_fmt(format_args!("{}", tac.emit_code())).unwrap();

        tac
    }

    #[test]
    fn test_basic_opt() {
        test_opt(
            "../testprogs/basic.c", 
            true, 
            true, 
            true,
            true,
            2,
        );        
    }

    #[test]
    fn test_basic() {
        test_opt(
            "../testprogs/basic.c", 
            false, 
            false, 
            false,
            false,
            0,
        );
    }

    #[test]
    fn test_control_flow_opt() {
        test_opt(
            "../testprogs/control_flow.c", 
            true, 
            true, 
            true,
            true,
            1,
        );
    }

    #[test]
    fn test_control_flow() {
        test_opt(
            "../testprogs/control_flow.c", 
            false, 
            false, 
            false,
            false,
            0,
        );
    }
}