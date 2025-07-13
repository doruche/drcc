use crate::common::*;
use super::{
    Parser,
    Decl,
};

impl Parser {
    pub(super) fn decl(&mut self) -> Result<Decl> {
        let type_token = self.eat_current();
        if !type_token.is_type() {
            return Err(Error::Parse(format!("Expected a type token, found {:?}", type_token)));
        }
        let id_token = self.eat(TokenType::Identifier, "Expected an identifier")?;
        if self.is_at_end() {
            return Err(Error::Parse("Unexpected end of input while parsing declaration.".into()));
        }

        match self.peek().unwrap().get_type() {
            TokenType::LParen => {
                // function
                self.eat(TokenType::LParen, "Expected '(' after function name.")?;
                // currentlt, void is expected.
                self.eat(TokenType::Void, "Expected 'void' for function parameters.")?;
                self.eat(TokenType::RParen, "Expected ')' after function parameters.")?;
                self.eat(TokenType::LBrace, "Expected '{' to start function body.");

                let mut body = vec![];
                while !self.is_at_end() && self.peek().unwrap().get_type() != TokenType::RBrace {
                    body.push(self.block_item()?);
                }

                self.eat(TokenType::RBrace, "Expected '}' to end function body.")?;
                Ok(Decl::FuncDecl {
                    return_type: (DataType::Int, type_token.span),
                    name: (id_token.inner.as_identifier(), id_token.span),
                    body,
                })
            },
            _ => {
                // variable
                todo!()
            }
        }
    }
}