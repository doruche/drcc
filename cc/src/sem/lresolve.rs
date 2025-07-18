use crate::common::*;
use super::{
    Parser,
    SymbolTable,
    SymError,
    StaticVarSymbol,
    Function,
    FuncSymbol,
    TopLevel,
    LocalVarDecl,
    BlockItem,
    Stmt,
    ForInit,
    TypedExpr,
    Expr,
    UnaryOp,
    BinaryOp,
};

pub(super) struct LResolver {
    pub label_counter: usize,
    pub loop_labels: Vec<usize>,
}

impl LResolver {
    pub fn new() -> Self {
        Self {
            label_counter: 0,
            loop_labels: vec![],
        }
    }

    fn alloc_label(&mut self) -> usize {
        let label = self.label_counter;
        self.label_counter += 1;
        label
    }

    pub fn resolve_func(
        &mut self,
        func: &mut Function,
    ) -> Result<()> {
        if func.body.is_none() {
            return Ok(());
        }
        self.label_counter = 0;
        self.loop_labels.clear();
        for item in func.body
            .as_mut()
            .unwrap()
            .iter_mut()
        {
            self.resolve_block_item(item)?;  
        }

        Ok(())
    }

    pub fn resolve_block_item(
        &mut self,
        item: &mut BlockItem,
    ) -> Result<()> {
        match item {
            BlockItem::Declaration(..) => {},
            BlockItem::Statement(stmt) => self.resolve_stmt(stmt)?,
        }
        Ok(())
    }

    pub fn resolve_stmt(
        &mut self,
        stmt: &mut Stmt,
    ) -> Result<()> {
        match stmt {
            Stmt::Compound(items) => {
                for item in items {
                    self.resolve_block_item(item)?;
                }
            },
            Stmt::While { body, loop_label, .. } |
            Stmt::DoWhile { body, loop_label, .. } |
            Stmt::For { body, loop_label, .. } => {
                *loop_label = self.alloc_label();
                self.loop_labels.push(*loop_label);
                self.resolve_stmt(body.as_mut())?;
                assert!(self.loop_labels.pop().is_some(), "Loop label stack underflow");
            },
            Stmt::Continue { loop_label, span } |
            Stmt::Break { loop_label, span } => {
                if let Some(&last_label) = self.loop_labels.last() {
                    *loop_label = last_label;
                } else {
                    return Err(Error::semantic(
                        "'continue' or 'break' must be inside a loop".to_string(),
                        *span,
                    ));
                }
            },
            Stmt::If { then_branch, else_branch, .. } => {
                self.resolve_stmt(then_branch.as_mut())?;
                if let Some(else_branch) = else_branch {
                    self.resolve_stmt(else_branch.as_mut())?;
                }
            },
            _ => {},
        }

        Ok(())
    }
}



// pub(super) fn lresolve_stmt(
//     &mut self,
//     stmt: Stmt,
// ) -> Result<()> {
//     match stmt {
//         Stmt::While { span, controller, body, .. } => {
//             let loop_label = self.label_counter;
//             self.label_counter += 1;
//             self.loop_labels.push(loop_label);
//             let r_body = self.lresolve_stmt(*body)?;
//             self.loop_labels.pop().ok_or(Error::semantic(
//                 "Loop label stack underflow".to_string(),
//                 span,
//             ))?;
//             Ok(Stmt::While {
//                 span,
//                 controller,
//                 body: Box::new(r_body),
//                 loop_label,
//             })
//         },
//         Stmt::DoWhile { span, body, controller, loop_label } => {
//             let loop_label = self.label_counter;
//             self.label_counter += 1;
//             self.loop_labels.push(loop_label);
//             let r_body = self.lresolve_stmt(*body)?;
//             self.loop_labels.pop().ok_or(Error::semantic(
//                 "Loop label stack underflow".to_string(),
//                 span,
//             ))?;
//             Ok(Stmt::DoWhile {
//                 span,
//                 body: Box::new(r_body),
//                 controller,
//                 loop_label,
//             })
//         },
//         Stmt::For { span, initializer, controller, post, body, .. } => {
//             let loop_label = self.label_counter;
//             self.label_counter += 1;
//             self.loop_labels.push(loop_label);
//             let r_body = self.lresolve_stmt(*body)?;
//             self.loop_labels.pop().ok_or(Error::semantic(
//                 "Loop label stack underflow".to_string(),
//                 span,
//             ))?;
//             Ok(Stmt::For {
//                 span,
//                 initializer,
//                 controller,
//                 post,
//                 body: Box::new(r_body),
//                 loop_label,
//             })
//         }
//         Stmt::Continue { span, loop_label } => {
//             let loop_label = self.loop_labels
//                 .last()
//                 .copied()
//                 .ok_or(Error::semantic("'continue' must be inside a loop", span))?;
//             Ok(Stmt::Continue { span, loop_label })
//         },
//         Stmt::Break { span, loop_label } => {
//             let loop_label = self.loop_labels
//                 .last()
//                 .copied()
//                 .ok_or(Error::semantic("'break' must be inside a loop", span))?;
//             Ok(Stmt::Break { span, loop_label })
//         },
//         Stmt::If { condition, then_branch, else_branch } => {
//             let r_then_branch = self.lresolve_stmt(*then_branch)?;
//             let r_else_branch = else_branch.map(|b| self.lresolve_stmt(*b)).transpose()?;
//             Ok(Stmt::If {
//                 condition: condition,
//                 then_branch: Box::new(r_then_branch),
//                 else_branch: r_else_branch.map(Box::new),
//             })
//         },
//         Stmt::Compound(items) => {
//             let mut r_items = vec![];
//             for item in items {
//                 r_items.push(self.lresolve_block_item(item)?);
//             }
//             Ok(Stmt::Compound(r_items))
//         },
//         e@Stmt::Expr(_) => Ok(e),
//         n@Stmt::Nil => Ok(n),
//         r@Stmt::Return{..} => Ok(r),
//     }
// }
// 