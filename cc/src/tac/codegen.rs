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
                    decls.push(Function {
                        name: name.0,
                        return_type: return_type.0,
                        body: body_insns,
                    });
                },
                _ => unimplemented!(),
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
        HirBlockItem::Declaration(decl) => parse_decl(decl, next_temp_id, next_label_id),
        HirBlockItem::Statement(stmt) => parse_stmt(stmt, next_temp_id, next_label_id),
    }
}

pub(super) fn parse_decl(
    decl: HirDecl,
    next_temp_id: &mut usize,
    next_label_id: &mut usize,
) -> Result<Vec<Insn>> {
    todo!()    
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
                Assign => todo!()
            }
        },
        HirExpr::Variable(name, span) => {
            todo!();
        },
        HirExpr::Assignment { span, left, right } => {
            todo!();
        }
    }
}