//! Intermediate-level optimizations.

mod cfg;
mod constant_folding;

use std::collections::HashMap;

use super::{
    TopLevel,
    CodeGen,
    Opt,
    Function,
    FuncContext,
    Insn,
    UnaryOp,
    BinaryOp,
    Operand,   
};

/// We only do intra-function optimizations.
impl CodeGen<Opt> {
    pub fn optimize(mut self, tac: TopLevel) -> TopLevel {
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

    pub fn opt_func(&mut self, func: Function) -> Function {
        if func.body.len() == 1 {
            // the ret insn we inserted
            return func;
        }

        let post_constant_folding = self.constant_folding(func);

        post_constant_folding
    }
}