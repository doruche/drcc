use std::collections::HashMap;
use std::hash::Hash;
use std::marker::PhantomData;

use crate::common::*;
use crate::sem::{
    HirTopLevel,
    HirFunction,
    HirLocalVarDecl,
    HirParam,
    HirStaticVar,
    HirBlockItem,
    HirStmt,
    HirForInit,
    HirTypedExpr,
    HirExpr,
    HirVariable,
    HirUnaryOp,
    HirBinaryOp,
};
use super::{
    Operand,
    Parse,
    Opt,
    FuncContext,
    CodeGen,
    Insn,
    StaticVar,
    LocalVar,
    Param,
    Function,
    TopLevel,
    UnaryOp,
    BinaryOp,
    LabelOperand,
    AutoGenLabel,
};

impl CodeGen<Parse> {
    pub fn parse(mut self, hir: HirTopLevel) -> (TopLevel, CodeGen<Opt>) {
        let mut functions = HashMap::new();
        let mut static_vars = HashMap::new();
        let strtb = hir.strtb;

        // static variables
        for (name, var) in hir.static_vars {    
            static_vars.insert(name, StaticVar {
                name,
                data_type: var.data_type,
                initializer: var.initializer,
                linkage: var.linkage,
            });
        }

        // functions
        for (name, function) in hir.funcs {
            let linkage = function.linkage;
            match function.body {
                None => {
                    let type_ = function.type_;
                    functions.insert(name, Function::Declared {
                        linkage,
                        name,
                        type_,
                    });
                },
                Some(insns) => {
                    let params = function.params
                        .into_iter()
                        .map(Param::from)
                        .collect::<Vec<_>>();
                    let return_type = function.type_.return_type;

                    self.cur_cx = Some(FuncContext {
                        local_vars: HashMap::new(),
                    });
                    let mut func_insns = vec![];
                    let next_temp_id = &mut 0;
                    let next_branch_label = &mut 0;
                    for item in insns {
                        let insn = self.parse_block_item(item, next_temp_id, next_branch_label);
                        func_insns.extend(insn);
                    }

                    // C standard specifies that a function without a return statement will
                    // return 0 if it is the main function, otherwise:
                    // 1. undefined behavior, if the value is used by the caller
                    // 2. works fine, if the value is not used by the caller
                    // hence, we insert a 'ret 0' instruction to make sure the standard is followed
                    func_insns.push(Insn::Return(Operand::Imm(Constant::Int(0))));

                    functions.insert(name, Function::Defined {
                        return_type,
                        linkage,
                        name,
                        params,
                        local_vars: self.cur_cx.take().unwrap().local_vars,
                        body: func_insns,
                    });

                    self.cur_cx = None;
                }
            }
        }

        (TopLevel {
            functions,
            static_vars,
            strtb,
        }, CodeGen {
            cur_cx: None,
            static_vars: self.static_vars,
            _stage: PhantomData,
        })
    }

    /// Returns the destination operand and any generated instructions.
    pub(super) fn parse_expr(
        &mut self,
        expr: HirTypedExpr, 
        next_temp_id: &mut usize,
        next_branch_label: &mut usize,
    ) -> (Operand, Option<Vec<Insn>>) {
        let type_ = expr.type_;
        let expr = expr.untyped;
        let dst = Operand::Temp(*next_temp_id, type_);
        *next_temp_id += 1;

        match expr {
            HirExpr::Cast { target, expr, .. } => {
                // now we only have int and long, so sext and trunc are enough
                let (src_operand, insns) = self.parse_expr(*expr, next_temp_id, next_branch_label);

                match (src_operand.data_type(), target) {
                    (a, b) if a == b => {
                        *next_temp_id -= 1;
                        return (src_operand, insns);
                    },
                    (DataType::Int, DataType::Long) => {
                        let insn = Insn::SignExt {
                            src: src_operand,
                            dst,
                        };
                        let insns = match insns {
                            Some(mut vec) => {
                                vec.push(insn);
                                vec
                            },
                            None => vec![insn],
                        };
                        return (dst, Some(insns));
                    },
                    (DataType::Long, DataType::Int) => {
                        let insn = Insn::Truncate {
                            src: src_operand,
                            dst,
                        };
                        let insns = match insns {
                            Some(mut vec) => {
                                vec.push(insn);
                                vec
                            },
                            None => vec![insn],
                        };
                        return (dst, Some(insns));
                    },
                    _ => unreachable!(),
                }
            },
            HirExpr::IntegerLiteral(val) => {
                (Operand::Imm(val), None)
            },
            HirExpr::Unary((op, span), expr) => {
                let (src, mut insns) = self.parse_expr(*expr, next_temp_id, next_branch_label);
                let insn = Insn::Unary {
                    op: op.into(),
                    src,
                    dst,
                };
                let insns = match insns {
                    Some(mut vec) => {
                        vec.push(insn);
                        Some(vec)
                    },
                    None => Some(vec![insn]),
                };
                (dst, insns)
            }
            HirExpr::Group(inner) => {
                self.parse_expr(*inner, next_temp_id, next_branch_label)
            }
            HirExpr::FuncCall { name, span, args } => {
                let mut top_insns = vec![];
                let mut arg_operands = vec![];
                for arg in args {
                    let (operand, insns) = self.parse_expr(arg, next_temp_id, next_branch_label);
                    if let Some(insns) = insns {
                        top_insns.extend(insns);
                    }
                    arg_operands.push(operand);
                }
                let insn = Insn::FuncCall {
                    target: name,
                    args: arg_operands,
                    dst,
                };
                top_insns.push(insn);
                (dst, Some(top_insns))
            },
            HirExpr::Binary { op: (op, ..), left, right } => {
                let op = op.into();
                use BinaryOp::*;
                match op {
                    And|Or => {
                        let mut top_insns = vec![];
                        let short_lable = LabelOperand::AutoGen(AutoGenLabel::Branch(*next_branch_label));
                        let end_lable = LabelOperand::AutoGen(AutoGenLabel::Branch(*next_branch_label + 1));
                        *next_branch_label += 2;

                        let (left_operand, left_insns) = self.parse_expr(*left, next_temp_id, next_branch_label);
                        if let Some(left_insns) = left_insns {
                            top_insns.extend(left_insns);
                        }
                        if let And = op {
                            top_insns.push(Insn::BranchIfZero {
                                src: left_operand,
                                label: short_lable,
                            });
                        } else {
                            top_insns.push(Insn::BranchNotZero {
                                src: left_operand,
                                label: short_lable,
                            });
                        }

                        let (right_operand, right_insns) = self.parse_expr(*right, next_temp_id, next_branch_label);
                        if let Some(right_insns) = right_insns {
                            top_insns.extend(right_insns);
                        }
                        if let And = op {
                            top_insns.push(Insn::BranchIfZero {
                                src: right_operand,
                                label: short_lable,
                            });
                        } else {
                            top_insns.push(Insn::BranchNotZero {
                                src: right_operand,
                                label: short_lable,
                            });
                        }

                        if let And = op {
                            top_insns.push(Insn::Move {
                                src: Operand::Imm(Constant::Int(1)),
                                dst,
                            });
                        } else {
                            top_insns.push(Insn::Move {
                                src: Operand::Imm(Constant::Int(0)),
                                dst,
                            });
                        }
                        top_insns.push(Insn::Jump(end_lable));
                        top_insns.push(Insn::Label(short_lable));
                        if let And = op {
                            top_insns.push(Insn::Move {
                                src: Operand::Imm(Constant::Int(0)),
                                dst,
                            });
                        } else {
                            top_insns.push(Insn::Move {
                                src: Operand::Imm(Constant::Int(1)),
                                dst,
                            });
                        }
                        top_insns.push(Insn::Label(end_lable));
                        (dst, Some(top_insns))
                    },
                    Add|Sub|Mul|Div|Rem|
                    Ls|Gt|GtEq|LsEq|Eq|NotEq => {
                        let (left_operand, mut left_insns) = self.parse_expr(*left, next_temp_id, next_branch_label);
                        let (right_operand, mut right_insns) = self.parse_expr(*right, next_temp_id, next_branch_label);
                        let insn = Insn::Binary {
                            op,
                            left: left_operand,
                            right: right_operand,
                            dst,
                        };
                        let mut insns = vec![];
                        if let Some(left_insns) = left_insns {
                            insns.extend(left_insns);
                        }
                        if let Some(right_insns) = right_insns {
                            insns.extend(right_insns);
                        }
                        insns.push(insn);
                        (dst, Some(insns))
                    },
                }
            },
            HirExpr::Var(var) => {
                // no need to allocate a temp id for variables.
                *next_temp_id -= 1;

                match var {
                    HirVariable::Local { name, local_id, data_type } => {
                        let operand = Operand::Var {
                            name,
                            local_id: Some(local_id),
                            data_type,
                        };
                        (operand, None)
                    },
                    HirVariable::Static { name, data_type } => {
                        self.static_vars.insert((name, data_type));
                        
                        let operand = Operand::Var {
                            name,
                            local_id: None,
                            data_type,
                        };
                        (operand, None)
                    }
                }
            },
            HirExpr::Assignment { span, left, right } => {
                *next_temp_id -= 1;

                let (left_operand, mut left_insns) = self.parse_expr(*left, next_temp_id, next_branch_label);
                let (right_operand, mut right_insns) = self.parse_expr(*right, next_temp_id, next_branch_label);

                let insn = Insn::Move {
                    src: right_operand,
                    dst: left_operand,
                };
                let mut insns = vec![];

                if let Some(right_insns) = right_insns {
                    insns.extend(right_insns);
                }
                if let Some(left_insns) = left_insns {
                    insns.extend(left_insns);
                }
                insns.push(insn);
                (left_operand, Some(insns))
            },
            HirExpr::Ternary { 
                condition, 
                then_expr, 
                else_expr,
                .. 
            } => {
                let mut top_insns = vec![];
                let (cond_operand, cond_insns) = self.parse_expr(*condition, next_temp_id, next_branch_label);
                if let Some(cond_insns) = cond_insns {
                    top_insns.extend(cond_insns);
                }

                let else_label = LabelOperand::AutoGen(AutoGenLabel::Branch(*next_branch_label));
                let end_label = LabelOperand::AutoGen(AutoGenLabel::Branch(*next_branch_label + 1));

                *next_branch_label += 2;
                top_insns.push(Insn::BranchIfZero {
                    src: cond_operand,
                    label: else_label,
                });
                let (then_operand, then_insns) = self.parse_expr(*then_expr, next_temp_id, next_branch_label);
                if let Some(then_insns) = then_insns {
                    top_insns.extend(then_insns);
                }
                top_insns.push(Insn::Move {
                    src: then_operand,
                    dst,
                });
                top_insns.push(Insn::Jump(end_label));
                top_insns.push(Insn::Label(else_label));
                let (else_operand, else_insns) = self.parse_expr(*else_expr, next_temp_id, next_branch_label);
                if let Some(else_insns) = else_insns {
                    top_insns.extend(else_insns);
                }
                top_insns.push(Insn::Move {
                    src: else_operand,
                    dst,
                });
                top_insns.push(Insn::Label(end_label));
                (dst, Some(top_insns))
            },
        }
    }

    pub(super) fn parse_stmt(
        &mut self,
        stmt: HirStmt, 
        next_temp_id: &mut usize,
        next_branch_label: &mut usize
    ) -> Vec<Insn> {
        let mut top_insns = vec![];
        match stmt {
            HirStmt::Return { expr , ..} => {
                let (operand, insns) = self.parse_expr(*expr, next_temp_id, next_branch_label);
                    let insn = Insn::Return(operand);
                    let insns = match insns {
                        Some(mut vec) => {
                            vec.push(insn);
                            vec
                        },
                        None => vec![insn],
                    };
                    top_insns.extend(insns);
            },
            HirStmt::Expr(expr) => {
                let (operand, insns) = self.parse_expr(*expr, next_temp_id, next_branch_label);
                if let Some(insns) = insns {
                    top_insns.extend(insns);
                }
            },
            HirStmt::If { condition, then_branch, else_branch } => {
                let (cond_operand, cond_insns) = self.parse_expr(*condition, next_temp_id, next_branch_label);
                if let Some(cond_insns) = cond_insns {
                    top_insns.extend(cond_insns);
                }
                match else_branch {
                    Some(else_branch) => {
                        let else_label = LabelOperand::AutoGen(AutoGenLabel::Branch(*next_branch_label));
                        let end_lable = LabelOperand::AutoGen(AutoGenLabel::Branch(*next_branch_label + 1));
                        *next_branch_label += 2;
                        top_insns.push(Insn::BranchIfZero {
                            src: cond_operand,
                            label: else_label,
                        });
                        let then_insns = self.parse_stmt(*then_branch, next_temp_id, next_branch_label);
                        top_insns.extend(then_insns);
                        top_insns.push(Insn::Jump(end_lable));
                        top_insns.push(Insn::Label(else_label));
                        let else_insns = self.parse_stmt(*else_branch, next_temp_id, next_branch_label);
                        top_insns.extend(else_insns);
                        top_insns.push(Insn::Label(end_lable));
                    },
                    None => {
                        let end_lable = LabelOperand::AutoGen(AutoGenLabel::Branch(*next_branch_label));
                        *next_branch_label += 1;
                        top_insns.push(Insn::BranchIfZero {
                            src: cond_operand,
                            label: end_lable,
                        });
                        let then_insns = self.parse_stmt(*then_branch, next_temp_id, next_branch_label);
                        top_insns.extend(then_insns);
                        top_insns.push(Insn::Label(end_lable));
                    }
                }
            },
            HirStmt::Compound(items) => {
                for item in items {
                    let insns = self.parse_block_item(item, next_temp_id, next_branch_label);
                    top_insns.extend(insns);
                }
            },
            HirStmt::Nil => {},
            HirStmt::Break { span, loop_label } => {
                top_insns.push(Insn::Jump(LabelOperand::AutoGen(AutoGenLabel::Break(loop_label))));
            }
            HirStmt::Continue { span, loop_label } => {
                top_insns.push(Insn::Jump(LabelOperand::AutoGen(AutoGenLabel::Continue(loop_label))));
            },
            HirStmt::While { span, controller, body, loop_label } => {
                let con_label = LabelOperand::AutoGen(AutoGenLabel::Continue(loop_label));
                let brk_label = LabelOperand::AutoGen(AutoGenLabel::Break(loop_label));

                top_insns.push(Insn::Label(con_label));
                let (ctrl_operand, ctrl_insns) = self.parse_expr(*controller, next_temp_id, next_branch_label);
                if let Some(ctrl_insns) = ctrl_insns {
                    top_insns.extend(ctrl_insns);
                }
                top_insns.push(Insn::BranchIfZero {
                    src: ctrl_operand,
                    label: brk_label,
                });
                let body_insns = self.parse_stmt(*body, next_temp_id, next_branch_label);
                top_insns.extend(body_insns);
                top_insns.push(Insn::Jump(con_label));
                top_insns.push(Insn::Label(brk_label));
            },
            HirStmt::DoWhile { span, body, controller, loop_label } => {
                let con_label = LabelOperand::AutoGen(AutoGenLabel::Continue(loop_label));
                let brk_label = LabelOperand::AutoGen(AutoGenLabel::Break(loop_label));
                let start_label = LabelOperand::AutoGen(AutoGenLabel::Branch(*next_branch_label));
                *next_branch_label += 1;

                top_insns.push(Insn::Label(start_label));
                let body_insns = self.parse_stmt(*body, next_temp_id, next_branch_label);
                top_insns.extend(body_insns);
                top_insns.push(Insn::Label(con_label));
                let (ctrl_operand, ctrl_insns) = self.parse_expr(*controller, next_temp_id, next_branch_label);
                if let Some(ctrl_insns) = ctrl_insns {
                    top_insns.extend(ctrl_insns);
                }
                top_insns.push(Insn::BranchNotZero {
                    src: ctrl_operand,
                    label: start_label,
                });
                top_insns.push(Insn::Label(brk_label));
            },
            HirStmt::For { 
                span, 
                initializer, 
                controller, 
                post, 
                body, 
                loop_label 
            } => {
                let con_label = LabelOperand::AutoGen(AutoGenLabel::Continue(loop_label));
                let brk_label = LabelOperand::AutoGen(AutoGenLabel::Break(loop_label));
                let start_label = LabelOperand::AutoGen(AutoGenLabel::Branch(*next_branch_label));
                *next_branch_label += 1;

                if let Some(init) = initializer {
                    match *init {
                        HirForInit::Declaration(decl) => {
                            let insns = self.parse_block_item(HirBlockItem::Declaration(decl), next_temp_id, next_branch_label);
                            top_insns.extend(insns);
                        },
                        HirForInit::Expression(expr) => {
                            let (_, insns) = self.parse_expr(*expr, next_temp_id, next_branch_label);
                            if let Some(insns) = insns {
                                top_insns.extend(insns);
                            }
                        }
                    }
                }
                top_insns.push(Insn::Label(start_label));
                if let Some(ctrl) = controller {
                    let (ctrl_operand, ctrl_insns) = self.parse_expr(*ctrl, next_temp_id, next_branch_label);
                    if let Some(ctrl_insns) = ctrl_insns {
                        top_insns.extend(ctrl_insns);
                    }
                    top_insns.push(Insn::BranchIfZero {
                        src: ctrl_operand,
                        label: brk_label,
                    });
                }
                let body_insns = self.parse_stmt(*body, next_temp_id, next_branch_label);
                top_insns.extend(body_insns);
                top_insns.push(Insn::Label(con_label));
                if let Some(post) = post {
                    let (_, insns) = self.parse_expr(*post, next_temp_id, next_branch_label);
                    if let Some(insns) = insns {
                        top_insns.extend(insns);
                    }
                }
                top_insns.push(Insn::Jump(start_label));
                top_insns.push(Insn::Label(brk_label));
            },
        }
        top_insns
    }

    pub(super) fn parse_block_item(
        &mut self,
        item: HirBlockItem,
        next_temp_id: &mut usize,
        next_branch_label: &mut usize,
    ) -> Vec<Insn> {
        match item {
            HirBlockItem::Declaration(local_var_decl) => {
                let HirLocalVarDecl {
                    name,
                    data_type,
                    local_id,
                    initializer,
                    ..
                } = local_var_decl;

                let var = Operand::Var {
                    name,
                    local_id: Some(local_id),
                    data_type,
                };

                let mut insns = vec![];
                if let Some(expr) = initializer {
                    let (src_operand, expr_insns) = self.parse_expr(expr, next_temp_id, next_branch_label);
                    if let Some(expr_insns) = expr_insns {
                        insns.extend(expr_insns);
                    }
                    insns.push(Insn::Move {
                        src: src_operand,
                        dst: var,
                    });
                }

                assert!(self.cur_cx.as_mut().unwrap().local_vars.insert(local_id, LocalVar {
                    name,
                    local_id,
                    data_type,
                }).is_none());

                insns
            },
            HirBlockItem::Statement(stmt) => self.parse_stmt(stmt, next_temp_id, next_branch_label),
        }

    }
}