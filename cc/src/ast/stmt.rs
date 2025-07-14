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
            _ => {
                // expression statement
                let expr = self.parse_expr()?;
                self.eat(TokenType::Semicolon, "Expected ';' after expression statement.")?;
                Ok(Stmt::Expr(Box::new(expr)))
            }
        }
    }

}