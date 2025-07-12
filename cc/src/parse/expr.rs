use crate::{common::{Error, Result, TokenType}, parse::Parser};


#[derive(Debug, Clone)]
pub enum Expr {
    IntegerLiteral(i64),
}

impl Parser {
    pub(super) fn primary(&mut self) -> Result<Expr> {
        let token = self.eat_current();
        match token.get_type() {
            TokenType::Integer => Ok(Expr::IntegerLiteral(token.inner.as_integer())),
            _ => Err(Error::Parse(format!(
                "Expected an integer literal, found: {:?}",
                token.get_type()
            ))),
        }
    }
}