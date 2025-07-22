//! First pass of LIR code generation.
//! Parses TAC and generates an incomplete LIR Top Level.

use std::collections::HashMap;
use std::marker::PhantomData;

use crate::asm::Register;
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
    TacLabelOperand,
    TacAutoGenLabel,
};
use super::{
    CodeGen,
    FuncContext,
    Canonic,
    Parse,
    LabelOperand,
    LabelSignature,
    TopLevel,  
    Function,
    StaticVar,
    DataSegment,
    BssSegment,
    Insn,
    IntermediateInsn,
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
            let type_ = get_functype(&func);
            let cx = FuncContext::new(name, type_);
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
            strtb,
        }, CodeGen {
            func_cxs: self.func_cxs,
            cur_func: self.cur_func,
            next_label: self.next_label,
            lmap: self.lmap,
            _stage: PhantomData,
        })
    }
}

impl CodeGen<Parse> {
    /// this will produce a function with incomplete instructions.
    /// it has no a complete prologue or epilogue, as we have to calculate
    /// the frame size through all stages.
    fn parse_function(
        &mut self,
        func: TacFunction,
    ) -> Function {
        let mut insns = vec![];
        let cx = self.cur_func
            .as_ref()
            .and_then(|name| self.func_cxs.get_mut(name))
            .expect("Internal error: Current function context not found");

        // prologue

        insns.push(Insn::Intermediate(IntermediateInsn::Prologue));

        for (i, param) in func.params.iter().enumerate() {
            let v_reg_id = cx.alloc_v_reg();
            let v_reg = Operand::VirtReg(v_reg_id);
            if i < 8 {
                insns.push(Insn::Mv(v_reg, Operand::PhysReg(Register::a(i))));
            } else {
                let size = param.data_type.size();
                let offset = get_param_offset(&func.params, i);
                // currenly we only have int and long, and no needs to consider the sign.
                let insn = match size {
                    4 => Insn::Lw(v_reg, Operand::Frame(offset)),
                    8 => Insn::Ld(v_reg, Operand::Frame(offset)),
                    _ => unreachable!(),
                };
                insns.push(insn);
            }
            cx.map_var2vreg(param.local_id, v_reg_id);
        }       

        let func_type = cx.type_.clone();

        let mut parsed_body = vec![];
        for insn in func.body {
            self.parse_insn(insn).map(|parsed_insns| {
                parsed_body.extend(parsed_insns);
            });
        }
        insns.extend(parsed_body);

        Function {
            name: func.name,
            linkage: func.linkage,
            func_type,
            body: insns,
            frame_size: 0, 
            callee_saved: vec![],
        }
    }

    fn parse_insn(
        &mut self,
        insn: TacInsn,
    ) -> Option<Vec<Insn>> {
        use Insn::*;

        // here we infer according operations directly from the type size.
        // cz we only have int and long now.
        // but later we should consider other types.
    
        let insns = match insn {
            TacInsn::Move { src, dst } => {
                let (src_op, _) = self.parse_operand(src);
                let (dst_op, _) = self.parse_operand(dst);
                vec![Mv(dst_op, src_op)]
            },
            TacInsn::Return(val) => {
                let (val_op, _) = self.parse_operand(val);
                vec![
                    Mv(Operand::PhysReg(Register::A0), val_op),
                    Intermediate(IntermediateInsn::Epilogue),
                ]
            },
            TacInsn::Unary { 
                op, 
                src, 
                dst 
            } => {
                if let TacUnaryOp::Pos = op {
                    return None;
                }
                let (src_op, type__) = self.parse_operand(src);
                let (dst_op, type_) = self.parse_operand(dst);
                assert_eq!(type_, type__);
                let size = type_.size();
                match (op, size) {
                    (TacUnaryOp::Not, 4) => vec![
                        Sextw(dst_op, src_op),
                        Seqz(dst_op, dst_op),
                    ],
                    (TacUnaryOp::Not, 8) => vec![Seqz(dst_op, src_op)],
                    (TacUnaryOp::Complement, _) => vec![Not(dst_op, src_op)],
                    (TacUnaryOp::Negate, 4) => vec![Negw(dst_op, src_op)],
                    (TacUnaryOp::Negate, 8) => vec![Neg(dst_op, src_op)],
                    _ => unreachable!(),
                }
            },
            TacInsn::Binary { 
                op, 
                left, 
                right, 
                dst 
            } => {
                let (left_op, left_type) = self.parse_operand(left);
                let (right_op, right_type) = self.parse_operand(right);
                let (dst_op, dst_type) = self.parse_operand(dst);
                assert_eq!(left_type, right_type);
                assert_eq!(left_type, dst_type);

                let size = left_type.size();
                match (op, size) {
                    (TacBinaryOp::Add, 4) => vec![Addw(dst_op, left_op, right_op)],
                    (TacBinaryOp::Add, 8) => vec![Add(dst_op, left_op, right_op)],
                    (TacBinaryOp::Sub, 4) => vec![Subw(dst_op, left_op, right_op)],
                    (TacBinaryOp::Sub, 8) => vec![Sub(dst_op, left_op, right_op)],
                    (TacBinaryOp::Mul, 4) => vec![Mulw(dst_op, left_op, right_op)],
                    (TacBinaryOp::Mul, 8) => vec![Mul(dst_op, left_op, right_op)],
                    (TacBinaryOp::Div, 4) => vec![Divw(dst_op, left_op, right_op)],
                    (TacBinaryOp::Div, 8) => vec![Div(dst_op, left_op, right_op)],
                    (TacBinaryOp::Rem, 4) => vec![Remw(dst_op, left_op, right_op)],
                    (TacBinaryOp::Rem, 8) => vec![Rem(dst_op, left_op, right_op)],
                    (TacBinaryOp::Eq, 4) => vec![
                        Sextw(left_op, left_op),
                        Sextw(right_op, right_op),
                        Sub(dst_op, left_op, right_op),
                        Seqz(dst_op, dst_op),
                    ],
                    (TacBinaryOp::Eq, 8) => vec![
                        Sub(dst_op, left_op, right_op),
                        Seqz(dst_op, dst_op),
                    ],
                    (TacBinaryOp::NotEq, 4) => vec![
                        Sextw(left_op, left_op),
                        Sextw(right_op, right_op),
                        Sub(dst_op, left_op, right_op),
                        Snez(dst_op, dst_op),
                    ],
                    (TacBinaryOp::NotEq, 8) => vec![
                        Sub(dst_op, left_op, right_op),
                        Snez(dst_op, dst_op),
                    ],
                    (TacBinaryOp::Ls, 4) => vec![
                        Sextw(left_op, left_op),
                        Sextw(right_op, right_op),
                        Slt(dst_op, left_op, right_op),
                    ],
                    (TacBinaryOp::Ls, 8) => vec![Slt(dst_op, left_op, right_op)],
                    (TacBinaryOp::Gt, 4) => vec![
                        Sextw(left_op, left_op),
                        Sextw(right_op, right_op),
                        Sgt(dst_op, left_op, right_op),
                    ],
                    (TacBinaryOp::Gt, 8) => vec![Sgt(dst_op, left_op, right_op)],
                    (TacBinaryOp::LsEq, 4) => vec![
                        Sextw(left_op, left_op),
                        Sextw(right_op, right_op),
                        Sgt(dst_op, left_op, right_op),
                        Seqz(dst_op, dst_op),
                    ],
                    (TacBinaryOp::LsEq, 8) => vec![
                        Sgt(dst_op, left_op, right_op),
                        Seqz(dst_op, dst_op),
                    ],
                    (TacBinaryOp::GtEq, 4) => vec![
                        Sextw(left_op, left_op),
                        Sextw(right_op, right_op),
                        Slt(dst_op, left_op, right_op),
                        Seqz(dst_op, dst_op),
                    ],
                    (TacBinaryOp::GtEq, 8) => vec![
                        Slt(dst_op, left_op, right_op),
                        Seqz(dst_op, dst_op),
                    ],
                    // these two operations are composed by other instructions.
                    (TacBinaryOp::And, _) | (TacBinaryOp::Or, _) => unreachable!(),
                    _ => unreachable!(),
                }
            },
            TacInsn::Jump(label) => {
                let signature = LabelSignature::from_tac(
                    self.cur_cx().name,
                    label,
                );
                let label_id = self.map_label(signature);
                vec![J(LabelOperand::AutoGen(label_id))]
            },
            TacInsn::BranchIfZero {
                src, 
                label 
            } => {
                let (src_op, _) = self.parse_operand(src);
                let signature = LabelSignature::from_tac(
                    self.cur_cx().name,
                    label,
                );
                let label_id = self.map_label(signature);
                vec![Beq(src_op, Operand::PhysReg(Register::Zero), LabelOperand::AutoGen(label_id))]
            },
            TacInsn::BranchNotZero { 
                src, 
                label 
            } => {
                let (src_op, _) = self.parse_operand(src);
                let signature = LabelSignature::from_tac(
                    self.cur_cx().name,
                    label,
                );
                let label_id = self.map_label(signature);
                vec![Bne(src_op, Operand::PhysReg(Register::Zero), LabelOperand::AutoGen(label_id))]
            },
            TacInsn::Label(label) => {
                let signature = LabelSignature::from_tac(
                    self.cur_cx().name,
                    label,
                );
                let label_id = self.map_label(signature); 
                vec![Label(LabelOperand::AutoGen(label_id))]
            },
            TacInsn::SignExt { src, dst } => {
                let (src_op, _) = self.parse_operand(src);
                let (dst_op, _) = self.parse_operand(dst);
                vec![Sextw(dst_op, src_op)]
            },
            TacInsn::Truncate { src, dst } => {
                // actually no need to implement this,
                // since if we need to operate on a smaller type,
                // we just use according instructions, say, addw instead of add.
                // however we have to insert a mv instruction as a placeholder,
                // as we allocate a virtual register for the destination during tac generation.
                // this instruction can be optimized out later.
                let (src_op, _) = self.parse_operand(src);
                let (dst_op, _) = self.parse_operand(dst);
                vec![Mv(dst_op, src_op)]
            },
            TacInsn::FuncCall { 
                target, 
                args, 
                dst 
            } => {
                let mut insns = vec![];
                let (dst_op, dst_type) = self.parse_operand(dst);
                let mut arg_ops = args.into_iter()
                    .map(|arg| self.parse_operand(arg))
                    .collect::<Vec<_>>();
                for i in 0..8 {
                    if i < arg_ops.len() {
                        insns.push(Mv(Operand::PhysReg(Register::a(i)), arg_ops[i].0));
                    }
                }
                let len = arg_ops.len();

                if len > 8 {
                    let stack_size = (arg_ops.len() - 8) * 8;
                    let padded_size = (stack_size + 15) & !15;
                    insns.push(Insn::Addi(
                        Operand::PhysReg(Register::Sp),
                        Operand::PhysReg(Register::Sp),
                        Operand::Imm(Constant::Long(-(padded_size as i64))),
                    ));
                    for (i, (op, type_)) in arg_ops.into_iter().enumerate().skip(8) {
                        let offset = (i - 8) * 8;
                        let insn = match type_.size() {
                            4 => Insn::Sw(op, Operand::Stack(offset as isize)),
                            8 => Insn::Sd(op, Operand::Stack(offset as isize)),
                            _ => unreachable!(),
                        };
                        insns.push(insn);
                    }
                }

                insns.push(Insn::Call(target));
                insns.push(Insn::Mv(dst_op, Operand::PhysReg(Register::A0)));

                if len > 8 {
                    let stack_size = (len - 8) * 8;
                    let padded_size = (stack_size + 15) & !15;
                    insns.push(Insn::Addi(
                        Operand::PhysReg(Register::Sp),
                        Operand::PhysReg(Register::Sp),
                        Operand::Imm(Constant::Long(padded_size as i64)),
                    ));
                }

                insns                
            }
        };

        Some(insns)
    }

    fn parse_operand(
        &mut self,
        operand: TacOperand,
    ) -> (Operand, DataType) {
        let cx = self.cur_cx_mut();
        
        let op = match operand {
            TacOperand::Imm(val) => Operand::Imm(val),
            TacOperand::Temp(temp_id, type_) => {
                let v_reg_id = cx.temp_vreg(temp_id)
                    .unwrap_or_else(|| {
                        let v_reg_id = cx.alloc_v_reg();
                        cx.map_temp2vreg(temp_id, v_reg_id);
                        v_reg_id
                    });
                Operand::VirtReg(v_reg_id)
            },
            TacOperand::Var { 
                name, 
                local_id, 
                data_type 
            } => {
                if let Some(local_id) = local_id {
                    let v_reg_id = cx.var_vreg(local_id)
                        .unwrap_or_else(|| {
                            let v_reg_id = cx.alloc_v_reg();
                            cx.map_var2vreg(local_id, v_reg_id);
                            v_reg_id
                        });
                    Operand::VirtReg(v_reg_id)
                } else {
                    Operand::Static(name)
                }
            }
        };

        (op, operand.data_type())
    }

    fn cur_cx(&self) -> &FuncContext {
        self.cur_func
            .as_ref()
            .and_then(|name| self.func_cxs.get(name))
            .expect("Internal error: Current function context not found")
    }

    fn cur_cx_mut(&mut self) -> &mut FuncContext {
        self.cur_func
            .as_ref()
            .and_then(|name| self.func_cxs.get_mut(name))
            .expect("Internal error: Current function context not found")
    }

    fn alloc_v_reg(&mut self) -> usize {
        let cx = self.cur_cx_mut();
        let v_reg = cx.alloc_v_reg();
        v_reg
    }

}

fn get_functype(func: &TacFunction) -> FuncType {
    FuncType {
        return_type: func.return_type,
        param_types: func.params.iter().map(|p| p.data_type).collect(),
    }
}

fn get_param_offset(
    params: &Vec<TacParam>,
    param_id: usize,
) -> isize {
    if param_id < 8 {
        panic!("Internal error: Parameter ID {} is less than 8, which is not supported", param_id);
    }
    if param_id >= params.len() {
        panic!("Internal error: Parameter ID {} is out of bounds for the number of parameters", param_id);
    }

    let mut cur_offset = 0;
    for i in 8..param_id {
        let prev_param = &params[i - 1];
        let param_size = prev_param.data_type.size();
        let slot_size = (param_size + 7) & !7;
        cur_offset += slot_size;
    }

    cur_offset as isize
}