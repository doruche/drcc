//! The first pass of semantic analysis.
//! Resolves names, fill in the symbol table, and builds an incomplete HirTopLevel.

use crate::{common::{
    Error, FuncType, InitVal, Linkage, Span, StorageClass, StrDescriptor
}, sem::symtb::CommonVar};
use super::{
    Parser,
    SymbolTable,
    SymError,
    StaticVarSymbol,
    StaticVar,
    FuncSymbol,
    TopLevel,
    Function,
    Param,
    LocalVarDecl,
    BlockItem,
    Stmt,
    ForInit,
    TypedExpr,
    Expr,
    UnaryOp,
    BinaryOp,
    Variable,
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
    ) -> Result<Option<LocalVarDecl>, (SymError, Span)> {
        match decl {
            AstDecl::FuncDecl {
                return_type,
                storage_class,
                name,
                params,
                body,
            } => {
                let functype = FuncType {
                    return_type,
                    param_types: params.iter()
                        .map(|param| param.data_type)
                        .collect(),
                };
                let mut r_params = vec![];
                let body = if let Some(body) = body {
                    self.symtb.ndef_func(
                        name.0,
                        functype.clone(),
                        storage_class,
                    ).map_err(|e| (e, name.1))?;
                    self.symtb.enter_block();

                    for param in &params {
                        let param_id = self.alloc_local_var();
                        self.symtb.ndef_var(param.name, param.data_type, Some(param_id))
                            .map_err(|e| (e, param.span))?;
                        r_params.push(Param {
                            name: param.name,
                            data_type: param.data_type,
                            local_id: param_id
                        });
                    }

                    let mut r_body = vec![];
                    for item in body {
                        if let Some(item) = self.nresolve_block_item(item)? {
                            r_body.push(item);
                        }
                    }
                    self.symtb.exit_block();
                    self.local_var_id_counter = 0;

                    Some(r_body)
                } else {
                    self.symtb.ndecl_func(
                        name.0,
                        functype.clone(),
                        storage_class,
                    ).map_err(|e| (e, name.1))?;
                    None
                };

                if let Some(prev) = self.functions.get_mut(&name.0) {
                    // in symtb.ndef_func/ndecl_func, we already checked
                    // the return type and parameter types, as well as linkage.
                    // so here we just update the body and params.
                    if body.is_some() {
                        prev.params = r_params;
                        prev.body = body;
                    }
                } else {
                    self.functions.insert(name.0, Function {
                        name: name.0,
                        type_: functype,
                        params: r_params,
                        linkage: match storage_class {
                            StorageClass::Static => Linkage::Internal,
                            _ => Linkage::External,
                        },
                        body,
                    });
                }

                Ok(None)
            },
            AstDecl::VarDecl { 
                name, 
                storage_class,
                span,
                data_type, 
                initializer 
            } => {
                // file-scope variables and block-scope variables should be handled differently.
                match self.symtb.nat_global_scope() {
                    true => {
                        if let Some(init) = &initializer {
                            if !init.is_constant() {
                                return Err((SymError::InvalidInitializer(name.0), name.1));
                            }
                        }

                        let () = self.symtb.ndef_static_var(
                            name.0, 
                            data_type, 
                            storage_class, 
                            initializer.is_some(),
                        ).map_err(|e| (e, name.1))?;

                        let initializer = initializer
                            .map(|expr| expr.to_constant());
    
                        let initializer =  if let Some(constant) = initializer {
                            InitVal::Const(constant)
                        } else if initializer.is_none() {
                            if let StorageClass::Extern = storage_class {
                                InitVal::None
                            } else {
                                InitVal::Tentative
                            }
                        } else {
                            return Err((SymError::InvalidInitializer(name.0), name.1));
                        };

                        if let Some(prev) = self.static_vars.get_mut(&name.0) {
                            if prev.data_type != data_type {
                                return Err((SymError::TypeMismatch {
                                    expected: prev.data_type,
                                    found: data_type,
                                }, name.1));
                            }
                            match (prev.linkage, storage_class) {
                                (Linkage::Internal, StorageClass::Unspecified) =>
                                    return Err((SymError::LinkageMismatch(name.0), name.1)),
                                (Linkage::External, StorageClass::Static) =>
                                    return Err((SymError::LinkageMismatch(name.0), name.1)),
                                _ => {}
                            }
                            match (prev.initializer, initializer) {
                                (InitVal::None, init) => 
                                    prev.initializer = init,
                                (InitVal::Tentative, InitVal::Const(_)) => 
                                    prev.initializer = initializer,
                                (InitVal::Const(_), InitVal::Const(_)) =>
                                    return Err((SymError::StaticVarRedefinition(name.0), name.1)),
                                _ => {}
                            }
                        } else {
                            self.static_vars.insert(name.0, StaticVar {
                                name: name.0,
                                data_type: data_type,
                                linkage: match storage_class {
                                    StorageClass::Static => Linkage::Internal,
                                    _ => Linkage::External,
                                },
                                initializer,
                            });
                        }

                        Ok(None)
                    },
                    false => {
                        if storage_class != StorageClass::Unspecified {
                            return Err((SymError::Unimplemented("Block-scope variable storage class".into()), name.1));
                        }
                        let local_id = self.alloc_local_var();
                        let () = self.symtb.ndef_var(name.0, data_type, Some(local_id))
                            .map_err(|e| (e, name.1))?;
                        let r_initializer = initializer
                            .map(|expr| self.nresolve_expr(*expr))
                            .transpose()?;
                        Ok(Some(LocalVarDecl {
                            name: name.0,
                            data_type,
                            span,
                            local_id,
                            initializer: r_initializer,
                        }))
                    }
                }
            }
        }
    }

    pub(super) fn nresolve_block_item(
        &mut self,
        item: AstBlockItem,
    ) -> Result<Option<BlockItem>, (SymError, Span)> {
        match item {
            AstBlockItem::Declaration(decl) => self
                .nresolve_decl(decl)
                .map(|decl| decl.map(BlockItem::Declaration)),
            AstBlockItem::Statement(stmt) => self
                .nresolve_stmt(stmt)
                .map(|stmt| Some(BlockItem::Statement(stmt))),
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
                    if let Some(item) = self.nresolve_block_item(item)? {
                        r_items.push(item);
                    }
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
                let var = self.symtb.nlookup_var(name)
                    .map_err(|sym_e| (sym_e, span))?;
                if let Some(local_id) = var.local_id {
                    Ok(Expr::Var(Variable::Local { name, local_id, data_type: var.data_type }))
                } else {
                    Ok(Expr::Var(Variable::Static { name, data_type: var.data_type }))
                }
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
                    let var = self.symtb.nlookup_var(name)
                        .map_err(|sym_e| (sym_e, span))?;
                    let var = if let Some(local_id) = var.local_id {
                        Variable::Local { name, local_id, data_type: var.data_type }
                    } else {
                        Variable::Static { name, data_type: var.data_type }
                    };
                    let right = self.nresolve_expr(*right)?;
                    Ok(Expr::Assignment {
                        span,
                        left: Box::new(TypedExpr::untyped(Expr::Var(var))),
                        right: Box::new(right),
                    })
                } else {
                    Err((SymError::InvalidLValue, span))
                }
            },
            AstExpr::Ternary { condition, then_expr, else_expr, span } => {
                let condition = self.nresolve_expr(*condition)?;
                let then_expr = self.nresolve_expr(*then_expr)?;
                let else_expr = self.nresolve_expr(*else_expr)?;
                Ok(Expr::Ternary {
                    span,
                    condition: Box::new(condition),
                    then_expr: Box::new(then_expr),
                    else_expr: Box::new(else_expr),
                })
            }
            AstExpr::FuncCall { name, span, args } => {
                let () = self.symtb.ncheck_func(name)
                    .map_err(|sym_e| (sym_e, span))?;

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
            AstExpr::Cast { target, expr, span } => {
                // We'll check whether the cast is valid in the type checking pass.
                let expr = self.nresolve_expr(*expr)?;
                Ok(Expr::Cast {
                    target,
                    expr: Box::new(expr),
                    span,
                })
            }
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
                if decl.is_none() {
                    panic!("Internal error: function declaration should not be passed to 'nresolve_for_init'");
                }
                Ok(ForInit::Declaration(decl.unwrap()))
            },
            AstForInit::Expression(expr) => {
                let expr = self.nresolve_expr(expr)?;
                Ok(ForInit::Expression(Box::new(expr)))
            }
        }
    }
}