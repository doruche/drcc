//! The first pass of semantic analysis.
//! Resolves names, fill in the symbol table, and builds an incomplete HirTopLevel.

use crate::common::{
    StrDescriptor,
    Span,
    Error,
    FuncType,
    Linkage,
};
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
use crate::ast::{
    AstTopLevel,
    AstDecl,
    AstBlockItem,
    AstStmt,
    AstExpr,
    AstForInit,
    AstUnaryOp,
    AstBinaryOp,
};

impl Parser {
    pub(super) fn nresolve_decl(
        &mut self,
        decl: AstDecl,
    ) -> Result<Decl, (SymError, Span)> {
        match decl {
            AstDecl::FuncDecl {
                return_type,
                name,
                params,
                body,
            } => {
                let functype = FuncType {
                    return_type: return_type.0,
                    param_types: params.iter()
                        .map(|param| param.data_type)
                        .collect(),
                };

                let body = if let Some(body) = body {
                    self.symtb.ndef_func(
                        name.0,
                        functype,
                        Linkage::External,
                    ).map_err(|e| (e, name.1))?;
                    self.symtb.enter_block();

                    for param in &params {
                        self.symtb.ndef_var(param.name)
                            .map_err(|e| (e, param.span))?;
                    }

                    let mut r_body = vec![];
                    for item in body {
                        r_body.push(self.nresolve_block_item(item)?);
                    }
                    self.symtb.exit_block();
                    Some(r_body)
                } else {
                    self.symtb.ndecl_func(
                        name.0,
                        functype,
                        Linkage::External,
                    ).map_err(|e| (e, name.1))?;
                    None
                };
                
                Ok(Decl::FuncDecl {
                    return_type,
                    name,
                    params: params.into_iter()
                        .map(|param| param.into())
                        .collect(),
                    body,
                })
            },
            AstDecl::VarDecl { 
                name, 
                data_type, 
                initializer 
            } => {
                self.symtb.ndef_var(name.0)
                    .map_err(|e| (e, name.1))?;
                let r_initializer = initializer
                    .map(|expr| self.nresolve_expr(*expr))
                    .transpose()?
                    .map(Box::new);

                Ok(Decl::VarDecl {
                    name,
                    data_type,
                    initializer: r_initializer,
                })
            }
        }
    }

    pub(super) fn nresolve_block_item(
        &mut self,
        item: AstBlockItem,
    ) -> Result<BlockItem, (SymError, Span)> {
        match item {
            AstBlockItem::Declaration(decl) => self
                .nresolve_decl(decl)
                .map(BlockItem::Declaration),
            AstBlockItem::Statement(stmt) => self
                .nresolve_stmt(stmt)
                .map(BlockItem::Statement),
        }
    }
    
    pub(super) fn nresolve_stmt(
        &mut self,
        stmt: AstStmt,
    ) -> Result<Stmt, (SymError, Span)> {
        match stmt {
            AstStmt::Nil => Ok(Stmt::Nil),
            AstStmt::Expr(expr) => {
                let expr = self.nresolve_expr(*expr)?;
                Ok(Stmt::Expr(Box::new(expr)))
            },
            AstStmt::Return { span, expr } => {
                let expr = self.nresolve_expr(*expr)?;
                Ok(Stmt::Return {
                    span,
                    expr: Box::new(expr),
                })
            },
            AstStmt::If { condition, then_branch, else_branch} => {
                let condition = self.nresolve_expr(*condition)?;
                let then_branch = self.nresolve_stmt(*then_branch)?;
                let else_branch = else_branch
                    .map(|stmt| self.nresolve_stmt(*stmt))
                    .transpose()?;
                Ok(Stmt::If {
                    condition: Box::new(condition),
                    then_branch: Box::new(then_branch),
                    else_branch: else_branch.map(Box::new),
                })
            },
            AstStmt::Compound(items) => {
                self.symtb.enter_block();
                let mut r_items = vec![];
                for item in items {
                    r_items.push(self.nresolve_block_item(item)?);
                }
                self.symtb.exit_block();
                Ok(Stmt::Compound(r_items))
            },
            AstStmt::Break(span) => 
                Ok(Stmt::Break { span, loop_label: usize::MAX }),
            AstStmt::Continue(span) =>
                Ok(Stmt::Continue { span, loop_label: usize::MAX }),
            AstStmt::While { span, controller, body } => {
                let controller = self.nresolve_expr(*controller)?;
                let body = self.nresolve_stmt(*body)?;
                Ok(Stmt::While {
                    span,
                    controller: Box::new(controller),
                    body: Box::new(body),
                    loop_label: usize::MAX,
                })
            },
            AstStmt::DoWhile { span, body, controller } => {
                let body = self.nresolve_stmt(*body)?;
                let controller = self.nresolve_expr(*controller)?;
                Ok(Stmt::DoWhile {
                    span,
                    body: Box::new(body),
                    controller: Box::new(controller),
                    loop_label: usize::MAX,
                })
            },
            AstStmt::For { span, initializer, controller, post, body } => {
                self.symtb.enter_block();
                let r_initializer = initializer
                    .map(|init| self.nresolve_for_init(*init))
                    .transpose()?
                    .map(Box::new);
                let r_controller = controller
                    .map(|expr| self.nresolve_expr(*expr))
                    .transpose()?
                    .map(Box::new);
                let r_post = post
                    .map(|expr| self.nresolve_expr(*expr))
                    .transpose()?
                    .map(Box::new);
                let body = self.nresolve_stmt(*body)?;
                self.symtb.exit_block();
                Ok(Stmt::For {
                    span,
                    initializer: r_initializer,
                    controller: r_controller,
                    post: r_post,
                    body: Box::new(body),
                    loop_label: usize::MAX,
                })
            },
        }
    }

    pub(super) fn nresolve_expr(
        &mut self,
        expr: AstExpr,
    ) -> Result<TypedExpr, (SymError, Span)> {
        let inner = match expr {
            AstExpr::IntegerLiteral(value) => Ok(Expr::IntegerLiteral(value)),
            AstExpr::Variable(name, span) => {
                let () = self.symtb.nlookup_var(name)
                    .map_err(|sym_e| (sym_e, span))?;
                Ok(Expr::Variable(name, span))
            },
            AstExpr::Unary((op, span), expr) => {
                let expr = self.nresolve_expr(*expr)?;
                Ok(Expr::Unary((op.into(), span), Box::new(expr)))
            },
            AstExpr::Binary { op: (op, span), left, right } => {
                let left = self.nresolve_expr(*left)?;
                let right = self.nresolve_expr(*right)?;
                Ok(Expr::Binary {
                    op: (op.into(), span),
                    left: Box::new(left),
                    right: Box::new(right),
                })
            },
            AstExpr::Group(expr) => {
                let expr = self.nresolve_expr(*expr)?;
                Ok(Expr::Group(Box::new(expr)))
            },
            AstExpr::Assignment { span, left, right } => {
                // currently, only variable assignment is supported.
                let left = *left;
                if let AstExpr::Variable(name, span) = left {
                    let _symbol = self.symtb.nlookup_var(name)
                        .map_err(|sym_e| (sym_e, span))?;
                    let right = self.nresolve_expr(*right)?;
                    Ok(Expr::Assignment {
                        span,
                        left: Box::new(TypedExpr::untyped(Expr::Variable(name, span))),
                        right: Box::new(right),
                    })
                } else {
                    Err((SymError::InvalidLValue, span))
                }
            },
            AstExpr::Ternary { condition, then_expr, else_expr } => {
                let condition = self.nresolve_expr(*condition)?;
                let then_expr = self.nresolve_expr(*then_expr)?;
                let else_expr = self.nresolve_expr(*else_expr)?;
                Ok(Expr::Ternary {
                    condition: Box::new(condition),
                    then_expr: Box::new(then_expr),
                    else_expr: Box::new(else_expr),
                })
            }
            AstExpr::FuncCall { name, span, args } => {
                // currently we only have int type, so no need to add a type checking pass.
                // we just check the number of arguments here, for now.
                let () = self.symtb.nlookup_func(name)
                    .map_err(|sym_e| (sym_e, span))?;
                let func = self.symtb.lookup_func(name)
                    .expect("Function should be defined in the symbol table.");
                if func.type_.param_types.len() != args.len() {
                    return Err((SymError::InvalidArguments(name), span));
                }

                let mut r_args = vec![];
                for arg in args {
                    let arg = self.nresolve_expr(arg)?;
                    r_args.push(arg);
                }
                Ok(Expr::FuncCall {
                    name,
                    span,
                    args: r_args,
                })
            },
        };
        Ok(TypedExpr::untyped(inner?))
    }

    fn nresolve_for_init(
        &mut self,
        init: AstForInit,
    ) -> Result<ForInit, (SymError, Span)> {
        match init {
            AstForInit::Declaration(decl) => {
                let decl = self.nresolve_decl(decl)?;
                Ok(ForInit::Declaration(decl))
            },
            AstForInit::Expression(expr) => {
                let expr = self.nresolve_expr(expr)?;
                Ok(ForInit::Expression(Box::new(expr)))
            }
        }
    }
}