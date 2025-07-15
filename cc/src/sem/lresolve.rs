use crate::common::*;
use super::{
    Parser,
    SymbolTable,
    SymError,
    VarSymbol,
    FuncSymbol,
    TopLevel,
    Decl,
    BlockItem,
    Stmt,
    ForInit,
    TypedExpr,
    Expr,
    UnaryOp,
    BinaryOp,
};

impl Parser {
    pub(super) fn lresolve_decl(
        &mut self,
        decl: Decl,
    ) -> Result<Decl> {
        match decl {
            v@Decl::VarDecl{..} => Ok(v),
            Decl::FuncDecl {
                return_type,
                name,
                body,
            } => {
                // reset label counter for each function
                self.label_counter = 0;
                self.loop_labels.clear();
                let mut r_body = vec![];
                for item in body {
                    r_body.push(self.lresolve_block_item(item)?);
                }
                assert!(self.loop_labels.is_empty(), "Loop labels should be empty after resolving function body.");
                Ok(Decl::FuncDecl {
                    return_type,
                    name,
                    body: r_body,
                })
            }
        }
    }

    pub(super) fn lresolve_block_item(
        &mut self,
        item: BlockItem,
    ) -> Result<BlockItem> {
        match item {
            // if the item is a declaration, then it must be a variable declaration,
            // so there is no need to resolve it further
            decl@BlockItem::Declaration(..) => Ok(decl),
            BlockItem::Statement(stmt) => Ok(BlockItem::Statement(self.lresolve_stmt(stmt)?)),
        }
    }

    pub(super) fn lresolve_stmt(
        &mut self,
        stmt: Stmt,
    ) -> Result<Stmt> {
        match stmt {
            Stmt::While { span, controller, body, .. } => {
                let loop_label = self.label_counter;
                self.label_counter += 1;
                self.loop_labels.push(loop_label);
                let r_body = self.lresolve_stmt(*body)?;
                self.loop_labels.pop().ok_or(Error::semantic(
                    "Loop label stack underflow".to_string(),
                    span,
                ))?;
                Ok(Stmt::While {
                    span,
                    controller,
                    body: Box::new(r_body),
                    loop_label,
                })
            },
            Stmt::DoWhile { span, body, controller, loop_label } => {
                let loop_label = self.label_counter;
                self.label_counter += 1;
                self.loop_labels.push(loop_label);
                let r_body = self.lresolve_stmt(*body)?;
                self.loop_labels.pop().ok_or(Error::semantic(
                    "Loop label stack underflow".to_string(),
                    span,
                ))?;
                Ok(Stmt::DoWhile {
                    span,
                    body: Box::new(r_body),
                    controller,
                    loop_label,
                })
            },
            Stmt::For { span, initializer, controller, post, body, .. } => {
                let loop_label = self.label_counter;
                self.label_counter += 1;
                self.loop_labels.push(loop_label);
                let r_body = self.lresolve_stmt(*body)?;
                self.loop_labels.pop().ok_or(Error::semantic(
                    "Loop label stack underflow".to_string(),
                    span,
                ))?;
                Ok(Stmt::For {
                    span,
                    initializer,
                    controller,
                    post,
                    body: Box::new(r_body),
                    loop_label,
                })
            }
            Stmt::Continue { span, loop_label } => {
                let loop_label = self.loop_labels
                    .last()
                    .copied()
                    .ok_or(Error::semantic("'continue' must be inside a loop", span))?;
                Ok(Stmt::Continue { span, loop_label })
            },
            Stmt::Break { span, loop_label } => {
                let loop_label = self.loop_labels
                    .last()
                    .copied()
                    .ok_or(Error::semantic("'break' must be inside a loop", span))?;
                Ok(Stmt::Break { span, loop_label })
            },
            Stmt::If { condition, then_branch, else_branch } => {
                let r_then_branch = self.lresolve_stmt(*then_branch)?;
                let r_else_branch = else_branch.map(|b| self.lresolve_stmt(*b)).transpose()?;
                Ok(Stmt::If {
                    condition: condition,
                    then_branch: Box::new(r_then_branch),
                    else_branch: r_else_branch.map(Box::new),
                })
            },
            Stmt::Compound(items) => {
                let mut r_items = vec![];
                for item in items {
                    r_items.push(self.lresolve_block_item(item)?);
                }
                Ok(Stmt::Compound(r_items))
            },
            e@Stmt::Expr(_) => Ok(e),
            n@Stmt::Nil => Ok(n),
            r@Stmt::Return{..} => Ok(r),
        }
    }
}