//! First pass of LIR code generation.
//! Parses TAC and generates an incomplete LIR Top Level.

use std::collections::HashMap;
use std::marker::PhantomData;

use crate::common::*;
use crate::tac::{
    TacTopLevel,
    TacFunction,
    TacStaticVar,
    TacInsn,
    TacParam,
    TacBinaryOp,
    TacUnaryOp,
    TacOperand,
    LabelOperand,
    AutoGenLabel,
};
use super::{
    CodeGen,
    FuncContext,
    Canonic,
    Parse,
    TopLevel,  
    Function,
    StaticVar,
    DataSegment,
    BssSegment,
    Insn,
    Operand,
};

impl CodeGen<Parse> {
    pub fn parse(mut self, tac: TacTopLevel) -> (TopLevel, CodeGen<Canonic>) {
        let mut functions = HashMap::new();
        let mut data_seg = DataSegment::new();
        let mut bss_seg = BssSegment::new();
        let strtb = tac.strtb;

        // Parse static variables
        for (name, var) in tac.static_vars {
            let static_var = StaticVar {
                name,
                data_type: var.data_type,
                initializer: var.initializer,
                linkage: var.linkage,
            };
            match var.initializer {
                InitVal::Tentative => 
                    bss_seg.items.push(static_var),
                InitVal::Const(c) if c.is_zero() =>
                    bss_seg.items.push(static_var),
                InitVal::Const(_) =>
                    data_seg.items.push(static_var),
                InitVal::None => {},
            }
        }

        // Parse functions
        for (name, func) in tac.functions {
            let type_ = parse_functype(&func);
            let cx = FuncContext {
                name,
                type_,
                next_v_reg: 0,
                frame_size: 0,
            };
            self.func_cxs.insert(name, cx);
            self.cur_func = Some(name);

            let parsed_func = self.parse_function(func);            
            functions.insert(name, parsed_func);

            self.cur_func = None;
        }

        (TopLevel {
            functions,
            bss_seg,
            data_seg,
        }, CodeGen {
            func_cxs: self.func_cxs,
            cur_func: self.cur_func,
            _stage: PhantomData,
        })
    }
}

impl CodeGen<Parse> {
    fn parse_function(
        &mut self,
        func: TacFunction,
    ) -> Function {
        todo!()
    }
}

fn parse_functype(func: &TacFunction) -> FuncType {
    FuncType {
        return_type: func.return_type,
        param_types: func.params.iter().map(|p| p.data_type).collect(),
    }
}