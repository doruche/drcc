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
            _ => {
                // expression statement
                let expr = self.parse_expr()?;
                self.eat(TokenType::Semicolon, "Expected ';' after expression statement.")?;
                Ok(Stmt::Expr(Box::new(expr)))
            }
        }
    }

}