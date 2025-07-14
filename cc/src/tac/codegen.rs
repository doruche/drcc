use crate::common::*;
use crate::sem::{
    HirTopLevel,
    HirDecl,
    HirBlockItem,
    HirStmt,
    HirTypedExpr,
    HirExpr,
    HirUnaryOp,
    HirBinaryOp,
};

use super::{
    Operand,
    Insn,
    Function,
    TopLevel,
    UnaryOp,
    BinaryOp,
};

#[derive(Debug)]
pub struct Parser {
    hir: HirTopLevel, 
}

impl Parser {
    pub fn new(hir: HirTopLevel) -> Self {
        Self {
            hir,
        }
    }

    pub fn parse(mut self) -> Result<TopLevel> {
        let mut decls = vec![];
        let strtb = self.hir.strtb;

        // global declarations
        for decl in self.hir.decls {
            match decl {
                HirDecl::FuncDecl { 
                    return_type, 
                    name, 
                    body 
                } => {
                    let mut body_insns = vec![];
                    let mut next_temp_id = 0;
                    let mut next_label_id = 0;
                    for stmt in body {
                        let insns = parse_block_item(stmt, &mut next_temp_id, &mut next_label_id)?;
                        body_insns.extend(insns);
                    }
                    // C standard specifies that a function without a return statement will
                    // return 0 if it is the main function, otherwise:
                    // 1. undefined behavior, if the value is used by the caller
                    // 2. works fine, if the value is not used by the caller
                    // here, we insert a 'ret 0' instruction to make sure the standard is followed
                    body_insns.push(Insn::Return(Some(Operand::Imm(0))));

                    decls.push(Function {
                        name: name.0,
                        return_type: return_type.0,
                        body: body_insns,
                    });
                },
                HirDecl::VarDecl{..} => unimplemented!(),
            }
        }

        Ok(TopLevel { functions: decls, strtb })
    }
}

pub(super) fn parse_block_item(
    item: HirBlockItem,
    next_temp_id: &mut usize,
    next_label_id: &mut usize,
) -> Result<Vec<Insn>> {
    match item {
        HirBlockItem::Declaration(decl) => {
            // this must be a variable declaration
            if let HirDecl::VarDecl {
                name, data_type, initializer,
            } = decl {
                // currentlt a pseudo implementation
                let var = Operand::Var(name.0);
                let mut insns = vec![];
                if let Some(expr) = initializer {
                    let (src_operand, expr_insns) = parse_expr(*expr, next_temp_id, next_label_id)?;
                    if let Some(expr_insns) = expr_insns {
                        insns.extend(expr_insns);
                    }
                    insns.push(Insn::Move {
                        src: src_operand,
                        dst: var,
                    });
                }
                Ok(insns)
            } else {
                unreachable!();
            }
        },
        HirBlockItem::Statement(stmt) => parse_stmt(stmt, next_temp_id, next_label_id),
    }
}

pub(super) fn parse_stmt(
    stmt: HirStmt, 
    next_temp_id: &mut usize,
    next_label_id: &mut usize
) -> Result<Vec<Insn>> {
    let mut top_insns = vec![];
    match stmt {
        HirStmt::Return { span, expr } => {
            let (operand, insns) = parse_expr(*expr, next_temp_id, next_label_id)?;
                let insn = Insn::Return(Some(operand));
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
            let (operand, insns) = parse_expr(*expr, next_temp_id, next_label_id)?;
            if let Some(insns) = insns {
                top_insns.extend(insns);
            }
        },
        HirStmt::If { condition, then_branch, else_branch } => {
            let (cond_operand, cond_insns) = parse_expr(*condition, next_temp_id, next_label_id)?;
            if let Some(cond_insns) = cond_insns {
                top_insns.extend(cond_insns);
            }
            match else_branch {
                Some(else_branch) => {
                    let else_label = *next_label_id;
                    let end_lable = else_label + 1;
                    *next_label_id += 2;
                    top_insns.push(Insn::BranchIfZero {
                        src: cond_operand,
                        label: else_label,
                    });
                    let then_insns = parse_stmt(*then_branch, next_temp_id, next_label_id)?;
                    top_insns.extend(then_insns);
                    top_insns.push(Insn::Jump(end_lable));
                    top_insns.push(Insn::Label(else_label));
                    let else_insns = parse_stmt(*else_branch, next_temp_id, next_label_id)?;
                    top_insns.extend(else_insns);
                    top_insns.push(Insn::Label(end_lable));
                },
                None => {
                    let end_lable = *next_label_id;
                    *next_label_id += 1;
                    top_insns.push(Insn::BranchIfZero {
                        src: cond_operand,
                        label: end_lable,
                    });
                    let then_insns = parse_stmt(*then_branch, next_temp_id, next_label_id)?;
                    top_insns.extend(then_insns);
                    top_insns.push(Insn::Label(end_lable));
                }
            }
        },
        HirStmt::Nil => {},
    }
    Ok(top_insns)
}

/// Returns the destination operand and any generated instructions.
pub(super) fn parse_expr(
    expr: HirTypedExpr, 
    next_temp_id: &mut usize,
    next_label_id: &mut usize,
) -> Result<(Operand, Option<Vec<Insn>>)> {
    let type_ = expr.type_;
    let expr = expr.expr;
    match expr {
        HirExpr::IntegerLiteral(val) => {
            Ok((Operand::Imm(val), None))
        },
        HirExpr::Unary((op, span), expr) => {
            let (src, mut insns) = parse_expr(*expr, next_temp_id, next_label_id)?;
            let temp_id = *next_temp_id;
            *next_temp_id += 1;
            let insn = Insn::Unary {
                op: op.into(),
                src,
                dst: Operand::Temp(temp_id),
            };
            let insns = match insns {
                Some(mut vec) => {
                    vec.push(insn);
                    Some(vec)
                },
                None => Some(vec![insn]),
            };
            Ok((Operand::Temp(temp_id), insns))
        }
        HirExpr::Group(inner) => {
            parse_expr(*inner, next_temp_id, next_label_id)
        }
        HirExpr::Binary { op: (op, span), left, right } => {
            let op = op.into();
            use BinaryOp::*;
            match op {
                And|Or => {
                    let mut top_insns = vec![];
                    let short_lable = *next_label_id;
                    let end_lable = *next_label_id + 1;
                    *next_label_id += 2;

                    let (left_operand, left_insns) = parse_expr(*left, next_temp_id, next_label_id)?;
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

                    let (right_operand, right_insns) = parse_expr(*right, next_temp_id, next_label_id)?;
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
                    let result_operand = Operand::Temp(*next_temp_id);
                    *next_temp_id += 1;
                    if let And = op {
                        top_insns.push(Insn::Move {
                            src: Operand::Imm(1),
                            dst: result_operand,
                        });
                    } else {
                        top_insns.push(Insn::Move {
                            src: Operand::Imm(0),
                            dst: result_operand,
                        });
                    }
                    top_insns.push(Insn::Jump(end_lable));
                    top_insns.push(Insn::Label(short_lable));
                    if let And = op {
                        top_insns.push(Insn::Move {
                            src: Operand::Imm(0),
                            dst: result_operand,
                        });
                    } else {
                        top_insns.push(Insn::Move {
                            src: Operand::Imm(1),
                            dst: result_operand,
                        });
                    }
                    top_insns.push(Insn::Label(end_lable));
                    Ok((result_operand, Some(top_insns)))
                },
                Add|Sub|Mul|Div|Rem|
                Ls|Gt|GtEq|LsEq|Eq|NotEq => {
                    let (left_operand, mut left_insns) = parse_expr(*left, next_temp_id, next_label_id)?;
                    let (right_operand, mut right_insns) = parse_expr(*right, next_temp_id, next_label_id)?;
                    let temp_id = *next_temp_id;
                    *next_temp_id += 1;
                    let insn = Insn::Binary {
                        op,
                        left: left_operand,
                        right: right_operand,
                        dst: Operand::Temp(temp_id),
                    };
                    let mut insns = vec![];
                    if let Some(left_insns) = left_insns {
                        insns.extend(left_insns);
                    }
                    if let Some(right_insns) = right_insns {
                        insns.extend(right_insns);
                    }
                    insns.push(insn);
                    Ok((Operand::Temp(temp_id), Some(insns)))
                },
                Assign => unreachable!(),
            }
        },
        HirExpr::Variable(name, span) => {
            Ok((
                Operand::Var(name),
                None,
            ))
        },
        HirExpr::Assignment { span, left, right } => {
            let (left_operand, mut left_insns) = parse_expr(*left, next_temp_id, next_label_id)?;
            let (right_operand, mut right_insns) = parse_expr(*right, next_temp_id, next_label_id)?;
            
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
            Ok((left_operand, Some(insns)))
        }
    }
}