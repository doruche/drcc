use crate::common::*;
use super::{
    Parser,
    TopLevel,
    Decl,
    Expr,
    Stmt,
    BlockItem,
    UnaryOp,
    BinaryOp,
    ForInit,
};

impl Parser {
    pub(super) fn stmt_top_level(&mut self) -> Result<Stmt> {
        self.statement()
    }

    fn statement(&mut self) -> Result<Stmt> {
        if self.is_at_end() {
            return Err(Error::Parse("Unexpected end of input".to_string()));
        }
        let token = self.peek().unwrap();
        match token.get_type() {
            TokenType::Return => {
                let return_token = self.eat(TokenType::Return, "Expected 'return' statement.")?;
                let expr = self.parse_expr()?;
                self.eat(TokenType::Semicolon, "Expected ';' after return expression.")?;
                Ok(Stmt::Return {
                    span: return_token.span,
                    expr: Box::new(expr),
                })
            },
            TokenType::Break => {
                let break_token = self.eat(TokenType::Break, "Expected 'break' statement.")?;
                self.eat(TokenType::Semicolon, "Expected ';' after 'break' statement.")?;
                Ok(Stmt::Break(break_token.span))
            },
            TokenType::Continue => {
                let continue_token = self.eat(TokenType::Continue, "Expected 'continue' statement.")?;
                self.eat(TokenType::Semicolon, "Expected ';' after 'continue' statement.")?;
                Ok(Stmt::Continue(continue_token.span))
            },
            TokenType::While => {
                let span = self.eat_current().span;
                self.eat(TokenType::LParen, "Expected '(' after 'while'.")?;
                let controller = Box::new(self.expr_top_level()?);
                self.eat(TokenType::RParen, "Expected ')' after 'while' condition.")?;
                let body = Box::new(self.stmt_top_level()?);
                Ok(Stmt::While {
                    span,
                    controller,
                    body,
                })
            },
            TokenType::Do => {
                let span = self.eat_current().span;
                let body = Box::new(self.stmt_top_level()?);
                self.eat(TokenType::While, "Expected 'while' after 'do' statement.")?;
                self.eat(TokenType::LParen, "Expected '(' after 'while'.")?;
                let controller = Box::new(self.expr_top_level()?);
                self.eat(TokenType::RParen, "Expected ')' after 'while' condition.")?;
                self.eat(TokenType::Semicolon, "Expected ';' after 'do-while' statement.")?;
                Ok(Stmt::DoWhile {
                    span,
                    body,
                    controller,
                })
            },
            TokenType::For => {
                let span = self.eat_current().span;
                self.eat(TokenType::LParen, "Expected '(' after ' for'.")?;
                let initializer = if self.peek().map_or(false, |t| t.get_type() == TokenType::Semicolon) {
                    self.eat_current();
                    None
                } else {
                    Some(Box::new(self.for_init()?))
                };

                let controller = if self.peek().map_or(false, |t| t.get_type() == TokenType::Semicolon) {
                    None
                } else {
                    Some(Box::new(self.expr_top_level()?))
                };
                self.eat(TokenType::Semicolon, "Expected ';' after 'for' condition.")?;

                let post = if self.peek().map_or(false, |t| t.get_type() == TokenType::RParen) {
                    None
                } else {
                    Some(Box::new(self.expr_top_level()?))
                };
                self.eat(TokenType::RParen, "Expected ')' after 'for' statement.")?;

                let body = Box::new(self.stmt_top_level()?);

                Ok(Stmt::For {
                    span,
                    initializer,
                    controller,
                    post,
                    body,
                })
            },
            TokenType::Semicolon =>  {
                self.eat_current();
                Ok(Stmt::Nil)
            },
            TokenType::If => {
                self.eat_current();
                self.eat(TokenType::LParen, "Expected '(' after 'if'.")?;
                let condition = Box::new(self.expr_top_level()?);
                self.eat(TokenType::RParen, "Expected ')' after 'if' condition.")?;
                let then_branch = Box::new(self.stmt_top_level()?);
                let else_branch = if self.peek().map_or(false, |t| t.get_type() == TokenType::Else) {
                    self.eat_current();
                    Some(Box::new(self.stmt_top_level()?))
                } else {
                    None
                };
                Ok(Stmt::If {
                    condition,
                    then_branch,
                    else_branch,
                })
            },
            TokenType::LBrace => {
                self.eat_current();
                let mut items = vec![];
                while !self.is_at_end() && self.peek().unwrap().get_type() != TokenType::RBrace {
                    items.push(self.block_item()?);
                }
                self.eat(TokenType::RBrace, "Expected '}' to close block statement.")?;
                Ok(Stmt::Compound(items))
            },
            _ => {
                // expression statement
                let expr = self.parse_expr()?;
                self.eat(TokenType::Semicolon, "Expected ';' after expression statement.")?;
                Ok(Stmt::Expr(Box::new(expr)))
            }
        }
    }

    fn for_init(&mut self) -> Result<ForInit> {
        if self.is_at_end() {
            return Err(Error::Parse("Unexpected end of input while parsing 'for' initializer.".into()));
        }

        if self.peek().unwrap().is_type() {
            // variable declaration
            Ok(ForInit::Declaration(self.var_decl()?))
        } else {
            // expression
            let expr = self.expr_top_level()?;
            self.eat(TokenType::Semicolon, "Expected ';' after 'for' initializer.")?;
            Ok(ForInit::Expression(expr))
        }
    }

}