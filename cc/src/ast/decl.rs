use crate::common::*;
use super::{
    Parser,
    Decl,
    Param,
};

impl Parser {
    pub(super) fn decl(&mut self) -> Result<Decl> {
        let type_token = self.eat_current();
        if !type_token.is_type() {
            return Err(Error::Parse(format!("Expected a type token, found {:?}", type_token)));
        }
        assert!(matches!(type_token.get_type(), TokenType::Int));

        let id_token = self.eat(TokenType::Identifier, "Expected an identifier")?;
        if self.is_at_end() {
            return Err(Error::Parse("Unexpected end of input while parsing declaration.".into()));
        }

        match self.peek().unwrap().get_type() {
            TokenType::LParen => {
                // function
                self.eat_current();
                
                let mut params = vec![];

                if self.peek().map_or(false, |t| t.get_type() == TokenType::Void)
                && self.peek_next().map_or(false, |t| t.get_type() == TokenType::RParen) {
                    self.eat_current();
                    self.eat_current();
                } else if self.peek().map_or(false, |t| t.get_type() == TokenType::RParen) {
                    self.eat_current();
                } else {
                    loop {
                        if self.is_at_end() {
                            return Err(Error::Parse("Unexpected end of input while parsing function parameters.".into()));
                        }
                        let next_token_type = self.peek().unwrap().get_type();
                        if next_token_type == TokenType::RParen {
                            self.eat_current();
                            break;
                        }

                        let param_type_token = self.eat_current();
                        if !param_type_token.is_type() {
                            return Err(Error::parse("Expected a type for function parameter.", param_type_token.span));
                        }

                        let id_token = self.eat(TokenType::Identifier, "Expected an identifier for function parameter")?;
                        if self.is_at_end() {
                            return Err(Error::Parse("Unexpected end of input while parsing function parameters.".into()));
                        }
                        if self.peek().unwrap().get_type() == TokenType::Comma {
                            self.eat_current();
                        }
                        params.push(Param {
                            name: id_token.inner.as_identifier(),
                            data_type: DataType::Int,
                            span: id_token.span,
                        });
                    }               
                }

                if self.is_at_end() {
                    return Err(Error::Parse("Unexpected end of input while parsing function body.".into()));
                }
                match self.peek().unwrap().get_type() {
                    TokenType::LBrace => {
                        self.eat_current();
                        let mut body = vec![];
                        while !self.is_at_end() && self.peek().unwrap().get_type() != TokenType::RBrace {
                            body.push(self.block_item()?);
                        }
                        if self.is_at_end() {
                            return Err(Error::parse("Unexpected end of input while parsing function body.", id_token.span));
                        }
                        self.eat(TokenType::RBrace, "Expected '}' to close function body.")?;
                        Ok(Decl::FuncDecl {
                            return_type: (DataType::Int, type_token.span),
                            name: (id_token.inner.as_identifier(), id_token.span),
                            params,
                            body: Some(body),
                        })
                    },
                    TokenType::Semicolon => {
                        self.eat_current();
                        Ok(Decl::FuncDecl {
                            return_type: (DataType::Int, type_token.span),
                            name: (id_token.inner.as_identifier(), id_token.span),
                            params,
                            body: None,
                        })
                    },
                    _ => Err(Error::parse("Expected '{' or ';' after function declaration.", id_token.span)),
                }
            },
            _ => {
                // variable
                match self.peek().unwrap().get_type() {
                    TokenType::Semicolon => {
                        self.eat_current();
                        Ok(Decl::VarDecl {
                            name: (id_token.inner.as_identifier(), id_token.span),
                            data_type: (DataType::Int, type_token.span),
                            initializer: None,
                        })
                    },
                    TokenType::Equal => {
                        self.eat_current();
                        let initializer = self.expr_top_level()?;
                        self.eat(TokenType::Semicolon, "Expected ';' after variable declaration.")?;
                        Ok(Decl::VarDecl {
                            name: (id_token.inner.as_identifier(), id_token.span),
                            data_type: (DataType::Int, type_token.span),
                            initializer: Some(Box::new(initializer)),
                        })
                    },
                    _ => Err(Error::parse("Expected ';' after variable declaration.", id_token.span)),
                }
            }
        }
    }

    pub(super) fn func_decl(&mut self) -> Result<Decl> {
        todo!()
    }

    pub(super) fn var_decl(&mut self) -> Result<Decl> {
        let type_token = self.eat_current();
        if !type_token.is_type() {
            return Err(Error::parse("Expected a type", type_token.span));
        }
        assert!(matches!(type_token.get_type(), TokenType::Int));

        let id_token = self.eat(TokenType::Identifier, "Expected an identifier")?;
        
        if self.is_at_end() {
            return Err(Error::Parse("Unexpected end of input while parsing variable declaration.".into()));
        }

        match self.peek().unwrap().get_type() {
            TokenType::Semicolon => {
                self.eat_current();
                Ok(Decl::VarDecl {
                    name: (id_token.inner.as_identifier(), id_token.span),
                    data_type: (DataType::Int, type_token.span),
                    initializer: None,
                })
            },
            TokenType::Equal => {
                self.eat_current();
                let initializer = self.expr_top_level()?;
                self.eat(TokenType::Semicolon, "Expected ';' after variable declaration.")?;
                Ok(Decl::VarDecl {
                    name: (id_token.inner.as_identifier(), id_token.span),
                    data_type: (DataType::Int, type_token.span),
                    initializer: Some(Box::new(initializer)),
                })
            },
            _ => Err(Error::parse("Expected ';' after variable declaration.", id_token.span)),
        }
    }
}