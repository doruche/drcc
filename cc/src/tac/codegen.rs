use crate::{ast::{self, Expr, Stmt}, common::Result, tac::{Function, Insn, Operand, TopLevel}};


#[derive(Debug)]
pub struct Parser {
    ast: ast::TopLevel,
    next_temp_id: usize,
}

impl Parser {
    pub fn new(ast: ast::TopLevel) -> Self {
        Self {
            ast,
            next_temp_id: 0,
        }
    }

    pub fn parse(mut self) -> Result<TopLevel> {
        let mut top_delcs = vec![];

        for decl in self.ast.decls {
            match decl {
                ast::TopDeclaration::FuncDecl { 
                    return_type, 
                    name, 
                    body 
                } => {
                    let mut body_insns = vec![];
                    for stmt in body {
                        let insns = parse_stmt(stmt, &mut self.next_temp_id)?;
                        body_insns.extend(insns);
                    }
                    top_delcs.push(Function {
                        name: name.0,
                        return_type: return_type.0,
                        body: body_insns,
                    });
                },
                _ => unimplemented!(),
            }
        }

        Ok(TopLevel { functions: top_delcs })
    }
}

pub(super) fn parse_stmt(stmt: Stmt, next_temp_id: &mut usize) -> Result<Vec<Insn>> {
    let mut top_insns = vec![];
    match stmt {
        Stmt::Return { span, expr } => {
            let (operand, insns) = parse_expr(*expr, next_temp_id)?;
                let insn = Insn::Return(Some(operand));
                let insns = match insns {
                    Some(mut vec) => {
                        vec.push(insn);
                        vec
                    },
                    None => vec![insn],
                };
                top_insns.extend(insns);
            }
    }
    Ok(top_insns)
}

/// Returns the destination operand and any generated instructions.
pub(super) fn parse_expr(expr: Expr, next_temp_id: &mut usize) -> Result<(Operand, Option<Vec<Insn>>)> {
    match expr {
        Expr::IntegerLiteral(val) => {
            Ok((Operand::Imm(val), None))
        },
        Expr::Unary((op, span), expr) => {
            let (src, mut insns) = parse_expr(*expr, next_temp_id)?;
            let temp_id = *next_temp_id;
            *next_temp_id += 1;
            let insn = Insn::Unary {
                op,
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
        Expr::Group(inner) => {
            parse_expr(*inner, next_temp_id)
        }
        Expr::Binary { op: (op, span), left, right } => {
            let (left_operand, mut left_insns) = parse_expr(*left, next_temp_id)?;
            let (right_operand, mut right_insns) = parse_expr(*right, next_temp_id)?;
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
        }
    }
}