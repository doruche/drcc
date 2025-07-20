//! Final pass of semantic analysis.
//! Transforms the HirTopLevel into a fully typed and resolved structure.
use std::collections::HashMap;

use crate::common::*;
use super::{
    Parser,
    TopLevel,
    Function,
    StaticVar,
    ForInit,
    Param,
    LocalVarDecl,
    BlockItem,
    Stmt,
    TypedExpr,
    Expr,
    UnaryOp,
    BinaryOp,
    FuncSymbol,
    StaticVarSymbol,
};

pub struct TypeChecker<'a> {
    pub cur_return_type: Option<DataType>,

    // from parser.symtb
    pub func_defs: &'a HashMap<StrDescriptor, FuncSymbol>,
    pub strtb: &'a StringPool,
}

impl<'a> TypeChecker<'a> {
    pub fn new(
        func_defs: &'a HashMap<StrDescriptor, FuncSymbol>,
        strtb: &'a StringPool,
    ) -> Self {
        Self {
            func_defs,
            strtb,
            cur_return_type: None,
        }
    }

    pub fn type_static_var(
        &mut self,
        static_var: StaticVar,
    ) -> Result<StaticVar> {
        let mut static_var = static_var;
        match static_var.initializer {
            InitVal::None|InitVal::Tentative => Ok(static_var),
            InitVal::Const(constant) => {
                match (static_var.data_type, constant) {
                    (DataType::Int, Constant::Int(..)) |
                    (DataType::Long, Constant::Long(..)) => Ok(static_var),
                    (DataType::Long, Constant::Int(val)) => {
                        static_var.initializer = InitVal::Const(Constant::Long(val as i64));
                        Ok(static_var)
                    },
                    (DataType::Int, Constant::Long(val)) => {
                        static_var.initializer = InitVal::Const(Constant::Int(val as i32));
                        Ok(static_var)
                    },
                    _ => unreachable!(),
                }
            }
        }
    }

    pub fn type_function(
        &mut self,
        func: Function,
    ) -> Result<Function> {
        if func.body.is_none() {
            return Ok(func);
        }

        self.cur_return_type = Some(func.return_type);
        let mut func = func;
        
        let mut typed_body = vec![];
        for item in func.body.unwrap() {
            let typed_item = self.type_block_item(item)?;
            typed_body.push(typed_item);
        }
        func.body = Some(typed_body);

        self.cur_return_type = None;

        Ok(func)
    }

    pub fn type_block_item(
        &mut self,
        item: BlockItem,
    ) -> Result<BlockItem> {
        match item {
            BlockItem::Declaration(mut decl) => {
                let typed_decl = self.type_local_var_decl(decl)?;
                Ok(BlockItem::Declaration(typed_decl))
            },
            BlockItem::Statement(stmt) =>
                Ok(BlockItem::Statement(self.type_stmt(stmt)?)),
        }
    }

    pub fn type_stmt(
        &mut self,
        stmt: Stmt,
    ) -> Result<Stmt> {
        match stmt {
            Stmt::Compound(block) => {       
                let mut typed_items = vec![];
                for item in block {
                    let typed_item = self.type_block_item(item)?;
                    typed_items.push(typed_item);
                }
                Ok(Stmt::Compound(typed_items))
            },
            Stmt::While { 
                body, 
                controller, 
                span, 
                loop_label 
            } => {
                let typed_body = self.type_stmt(*body)?;
                let typed_controller = self.type_expr(*controller)?;
                Ok(Stmt::While {
                    body: Box::new(typed_body),
                    controller: Box::new(typed_controller),
                    span,
                    loop_label,
                })
            },
            Stmt::DoWhile { 
                body, 
                controller, 
                span, 
                loop_label 
            } => {
                let typed_body = self.type_stmt(*body)?;
                let typed_controller = self.type_expr(*controller)?;
                Ok(Stmt::DoWhile {
                    body: Box::new(typed_body),
                    controller: Box::new(typed_controller),
                    span,
                    loop_label,
                })
            },
            Stmt::For {
                body,
                controller,
                initializer,
                post,
                loop_label,
                span,
            } => {
                let typed_body = self.type_stmt(*body)?;
                let typed_controller = controller.map(|c| {
                    self.type_expr(*c)
                }).transpose()?
                .map(Box::new);
                let typed_init = initializer.map(|init| {
                    self.type_for_init(*init)
                }).transpose()?
                .map(Box::new);
                let typed_post = post.map(|post| {
                    self.type_expr(*post)
                }).transpose()?
                .map(Box::new);

                Ok(Stmt::For {
                    body: Box::new(typed_body),
                    controller: typed_controller,
                    initializer: typed_init,
                    post: typed_post,
                    loop_label,
                    span,
                })
            }
            Stmt::Return { span, expr } => {
                let typed_expr = self.type_expr(*expr)?;
                let cur_return_type = self.cur_return_type
                    .expect("Internal error: cur_return_type should be set before type checking a return statement.");
                let unified_expr = try_cast(
                    cur_return_type,
                    typed_expr,
                    span,
                )?;
                Ok(Stmt::Return {
                    span,
                    expr: Box::new(unified_expr),
                })
            }
            Stmt::Expr(expr) => {
                let typed_expr = self.type_expr(*expr)?;
                Ok(Stmt::Expr(Box::new(typed_expr)))
            },
            Stmt::If { 
                condition, 
                then_branch, 
                else_branch 
            } => {
                let typed_condition = self.type_expr(*condition)?;
                let typed_then = self.type_stmt(*then_branch)?;
                let typed_else = if let Some(else_branch) = else_branch {
                    Some(Box::new(self.type_stmt(*else_branch)?))
                } else {
                    None
                };
                Ok(Stmt::If {
                    condition: Box::new(typed_condition),
                    then_branch: Box::new(typed_then),
                    else_branch: typed_else,
                })
            },
            _ => Ok(stmt),
        }
    }

    pub fn type_local_var_decl(
        &mut self,
        decl: LocalVarDecl,
    ) -> Result<LocalVarDecl> {
        // local variable declaration.
        // should cast the initializer if it exists and if necessary.
        if let Some(init) = decl.initializer {
            let typed_init = self.type_expr(init)?;
            let unified_init = try_cast(
                decl.data_type,
                typed_init,
                decl.span,
            )?;
            Ok(LocalVarDecl {
                data_type: decl.data_type,
                name: decl.name,
                local_id: decl.local_id,
                initializer: Some(unified_init),
                span: decl.span,
            })
        } else {
            Ok(decl)
        }
    }

    pub fn type_for_init(
        &mut self,
        init: ForInit,
    ) -> Result<ForInit> {
        match init {
            ForInit::Declaration(mut decl) => {
                let typed_decl = self.type_local_var_decl(decl)?;
                Ok(ForInit::Declaration(typed_decl))
            },
            ForInit::Expression(expr) => {
                let typed_expr = self.type_expr(*expr)?;
                Ok(ForInit::Expression(Box::new(typed_expr)))
            }
        }
    }

    pub fn type_expr(
        &mut self,
        expr: TypedExpr,
    ) -> Result<TypedExpr> {
        assert_eq!(expr.type_, DataType::Indeterminate, 
            "Internal error: type_expr should only be called on untyped expressions");

        let mut expr = expr;
        match expr.untyped {
            Expr::IntegerLiteral(constant) => match constant {
                Constant::Int(_) => {
                    expr.type_ = DataType::Int;
                    Ok(expr)
                },
                Constant::Long(_) => {
                    expr.type_ = DataType::Long;
                    Ok(expr)
                },
            },
            Expr::Var(var) => {
                expr.type_ = var.data_type();
                assert_ne!(expr.type_, DataType::Indeterminate,
                    "Internal error: Variable should have a determined type.");
                Ok(expr)
            },
            Expr::Cast { 
                target, 
                expr: expr_to_cast, 
                span 
            } => {
                let typed_expr = self.type_expr(*expr_to_cast)?;
                try_cast(target, typed_expr, span)
            },
            Expr::Group(inner_expr) => {
                let typed_inner = self.type_expr(*inner_expr)?;
                expr.type_ = typed_inner.type_;
                expr.untyped = Expr::Group(Box::new(typed_inner));
                Ok(expr)
            },
            Expr::Unary((op, span), inner_expr ) => {
                let typed_inner = self.type_expr(*inner_expr)?;

                // 1. for logical oprations (not, and, or...),  the results are always 'int' type.
                // but no cast is needed, as the tac code generated will only use the boolean value (0 or 1) directly.
                // 2. for unsigned values, negation will change the expression's type to signed,
                // but now we only hold signed types, so we just ignore them for now.
                let res_type = match op {
                    UnaryOp::Not => DataType::Int,
                    _ => typed_inner.type_,
                };

                Ok(TypedExpr {
                    untyped: Expr::Unary((op, span), Box::new(typed_inner)),
                    type_: res_type,
                })
            },
            Expr::Binary { 
                op: (op, span), 
                left, 
                right 
            } => {
                let typed_left = self.type_expr(*left)?;
                let typed_right = self.type_expr(*right)?;

                let super_type = match op {
                    BinaryOp::And|BinaryOp::Or => {
                        // no cast needed, directly return.
                        return Ok(TypedExpr {
                            untyped: Expr::Binary {
                                op: (op, span),
                                left: Box::new(typed_left),
                                right: Box::new(typed_right),
                            },
                            type_: DataType::Int,
                        });
                    }
                    _ => typed_left.type_.common(&typed_right.type_, span)?,
                };
                let unified_left = try_cast(super_type, typed_left, span)?;
                let unified_right = try_cast(super_type, typed_right, span)?;

                let unified_expr = Expr::Binary {
                    op: (op, span),
                    left: Box::new(unified_left),
                    right: Box::new(unified_right),
                };

                if op.is_arithmetic() {
                    // for arithmetic operations, types of results are the super type of the two operands.
                    Ok(TypedExpr {
                        untyped: unified_expr,
                        type_: super_type,
                    })
                } else {
                    // for comparison operations, the result is always 'int'.
                    Ok(TypedExpr {
                        untyped: unified_expr,
                        type_: DataType::Int,
                    })
                }
            },
            Expr::Assignment { 
                span, 
                left, 
                right 
            } => {
                let typed_left = self.type_expr(*left)?;
                let typed_right = self.type_expr(*right)?;
                let res_type = typed_left.type_;

                let unified_right = try_cast(
                    res_type,
                    typed_right,
                    span,
                )?;

                Ok(TypedExpr {
                    untyped: Expr::Assignment {
                        span,
                        left: Box::new(typed_left),
                        right: Box::new(unified_right),
                    },
                    type_: res_type,
                })
            },
            Expr::Ternary {
                condition, 
                then_expr, 
                else_expr, 
                span 
            } => {
                let typed_condition = self.type_expr(*condition)?;
                let typed_then = self.type_expr(*then_expr)?;
                let typed_else = self.type_expr(*else_expr)?;

                // mark: the condition may be converted to a int type?
                // we'll do it later if necessary.

                let super_type = typed_then.type_.common(&typed_else.type_, span)?;
                let unified_then = try_cast(super_type, typed_then, span)?;
                let unified_else = try_cast(super_type, typed_else, span)?;

                Ok(TypedExpr {
                    untyped: Expr::Ternary {
                        span,
                        condition: Box::new(typed_condition),
                        then_expr: Box::new(unified_then),
                        else_expr: Box::new(unified_else),
                    },
                    type_: super_type,
                })
            }
            Expr::FuncCall { 
                name, 
                span, 
                args 
            } => {
                let func = self.func_defs.get(&name)
                    .expect("Internal error: After name resolution, function should be defined in the symbol table.");
                let FuncType {
                    return_type,
                    param_types,
                } = &func.type_;

                if param_types.len() != args.len() {
                    return Err(Error::semantic(
                        format!("Function '{}' expects {} arguments, but got {}.", 
                            self.strtb.get(name).unwrap(),
                            param_types.len(), 
                            args.len()),
                        span,
                    ));
                }
                let mut unified_args = vec![];
                for (param_type, arg) in param_types.iter().zip(args) {
                    let typed_arg = self.type_expr(arg)?;
                    let unified_arg = try_cast(
                        *param_type,
                        typed_arg,
                        span,
                    )?;
                    unified_args.push(unified_arg);
                }
                Ok(TypedExpr {
                    untyped: Expr::FuncCall {
                        name,
                        span,
                        args: unified_args,
                    },
                    type_: *return_type,
                })
            }
        }
    }
}

pub(super) fn try_cast(
    target: DataType,
    expr: TypedExpr,
    span: Span,
) -> Result<TypedExpr> {
    match (target, expr.type_) {
        (DataType::Indeterminate, _) =>
            panic!("Internal error: Cannot cast to Indeterminate type"),
        (_, DataType::Indeterminate) =>
            panic!("Internal error: Cannot cast from Indeterminate type"),
        (a, b) if a == b => Ok(expr),
        (x, y) => {
            if x.common(&y, span).is_ok() {
                // x and y have a common type, so the cast is valid
                Ok(TypedExpr {
                    untyped: Expr::Cast {
                        target,
                        expr: Box::new(expr),
                        span,
                    },
                    type_: target,
                })
            } else {
                Err(Error::semantic(
                    format!("Cannot cast from {} to {}", y, x),
                    span,
                ))
            }
        }
    }        
}