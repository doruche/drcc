use crate::{ast::{Expr,Parser}, common::{DataType, Error, RawToken, Result, Span, StrDescriptor, Token, TokenType}};

#[derive(Debug, Clone)]
pub enum Stmt {
    Return {
        span: Span,
        expr: Box<Expr>,
    },
}

impl Parser {
    pub(super) fn stmt_top_level(&mut self) -> Result<Stmt> {
        self.statement()
    }

    fn statement(&mut self) -> Result<Stmt> {
        // currently, only return statements are supported
        let return_token = self.eat(TokenType::Return, "Expected 'return' statement.")?;
        let expr = self.parse_expr()?;
        self.eat(TokenType::Semicolon, "Expected ';' after return expression.")?;
        Ok(Stmt::Return {
            span: return_token.span,
            expr: Box::new(expr),
        })
    }

}