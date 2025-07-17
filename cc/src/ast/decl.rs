use crate::common::*;
use super::{
    Parser,
    Decl,
    Param,
};

impl Parser {
    pub(super) fn decl(&mut self) -> Result<Decl> {
        let mut types = vec![];
        let mut storage_class = None;
        while !self.is_at_end() {
            let next_token = self.peek().unwrap();
            if next_token.is_type() {
                types.push(self.eat_current());
            } else if next_token.is_specifier() {
                if storage_class.is_none() {
                    storage_class = Some(self.eat_current().to_storage_class());
                } else {
                    return Err(Error::parse("Only one storage class is allowed", next_token.span));
                }
            } else {
                break;
            }
        }
        if types.is_empty() {
            return Err(Error::Parse("Expected a type for declaration".into()));
        }
        // currentlt, only int type is supported
        if types.len() > 1 || !matches!(types[0].get_type(), TokenType::Int) {
            return Err(Error::Parse("Only 'int' type is supported for declaration".into()));
        }

        let type_token = types[0];

        let name_token = self.eat(TokenType::Identifier, "Expected an identifier for declaration")?;
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

                let linkage = (match storage_class {
                    Some(StorageClass::Static) => Linkage::Internal,
                    _ => Linkage::External,
                }, type_token.span);

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
                            return Err(Error::parse("Unexpected end of input while parsing function body.", name_token.span));
                        }
                        self.eat(TokenType::RBrace, "Expected '}' to close function body.")?;
                        Ok(Decl::FuncDecl {
                            return_type: (DataType::Int, type_token.span),
                            linkage,
                            name: (name_token.inner.as_identifier(), name_token.span),
                            params,
                            body: Some(body),
                        })
                    },
                    TokenType::Semicolon => {
                        self.eat_current();
                        Ok(Decl::FuncDecl {
                            return_type: (DataType::Int, type_token.span),
                            linkage,
                            name: (name_token.inner.as_identifier(), name_token.span),
                            params,
                            body: None,
                        })
                    },
                    _ => Err(Error::parse("Expected '{' or ';' after function declaration.", name_token.span)),
                }
            },
            _ => {
                // variable
                match self.peek().unwrap().get_type() {
                    TokenType::Semicolon => {
                        self.eat_current();
                        Ok(Decl::VarDecl {
                            name: (name_token.inner.as_identifier(), name_token.span),
                            storage_class: storage_class.map(|sc| (sc, type_token.span)),
                            data_type: (DataType::Int, type_token.span),
                            initializer: None,
                        })
                    },
                    TokenType::Equal => {
                        self.eat_current();
                        let initializer = self.expr_top_level()?;
                        self.eat(TokenType::Semicolon, "Expected ';' after variable declaration.")?;
                        Ok(Decl::VarDecl {
                            name: (name_token.inner.as_identifier(), name_token.span),
                            storage_class: storage_class.map(|sc| (sc, type_token.span)),
                            data_type: (DataType::Int, type_token.span),
                            initializer: Some(Box::new(initializer)),
                        })
                    },
                    _ => Err(Error::parse("Expected ';' after variable declaration.", name_token.span)),
                }
            }
        }
    }

    pub(super) fn func_decl(&mut self) -> Result<Decl> {
        todo!()
    }

    pub(super) fn var_decl(&mut self) -> Result<Decl> {
        let mut types = vec![];
        let mut storage_class = None;
        
        while !self.is_at_end() {
            let next_token = self.peek().unwrap();
            if next_token.is_type() {
                types.push(self.eat_current());
            } else if next_token.is_specifier() {
                if storage_class.is_none() {
                    storage_class = Some(self.eat_current().to_storage_class());
                } else {
                    return Err(Error::parse("Only one storage class is allowed", next_token.span));
                }
            } else {
                break;
            }
        }

        if types.is_empty() {
            return Err(Error::Parse("Expected a type for variable declaration".into()));
        }
        if types.len() > 1 || !matches!(types[0].get_type(), TokenType::Int) {
            return Err(Error::Parse("Only 'int' type is supported for variable declaration".into()));
        }

        let type_token = types[0];
        let id_token = self.eat(TokenType::Identifier, "Expected an identifier for variable declaration")?;
        if self.is_at_end() {
            return Err(Error::Parse("Unexpected end of input while parsing variable declaration.".into()));
        }

        match self.peek().unwrap().get_type() {
            TokenType::Semicolon => {
                self.eat_current();
                Ok(Decl::VarDecl {
                    name: (id_token.inner.as_identifier(), id_token.span),
                    storage_class: storage_class.map(|sc| (sc, type_token.span)),
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
                    storage_class: storage_class.map(|sc| (sc, type_token.span)),
                    data_type: (DataType::Int, type_token.span),
                    initializer: Some(Box::new(initializer)),
                })
            },
            _ => Err(Error::parse("Expected ';' after variable declaration.", id_token.span)),
        }
    }
}