use crate::{ast::{Expr, Stmt, TopDeclaration, TopLevel}, common::{DataType, Error, RawToken, Result, Token, TokenType}};

#[derive(Debug)]
pub struct Parser {
    input: Vec<Token>,
    position: usize,
    has_error: bool,
}

impl Parser {
    pub fn new(input: Vec<Token>) -> Self {
        Self {
            input,
            position: 0,
            has_error: false,
        }
    }

    pub fn parse_prog(&mut self) -> Result<TopLevel> {
        let mut decls = vec![];
        let mut errors = vec![];
        while !self.is_at_end() {
            match self.top_decl() {
                Ok(decl) => decls.push(decl),
                Err(e) => {
                    errors.push(e);
                    self.has_error = true;
                    self.synchronize();
                }
            }
        }
        if self.has_error {
            Err(Error::Errors(errors))
        } else {
            Ok(TopLevel { decls })
        }
    }

    pub fn parse_expr(&mut self) -> Result<Expr> {
        self.expr_top_level()
    }

    pub(super) fn is_at_end(&self) -> bool {
        self.position >= self.input.len()
    }

    pub(super) fn peek(&self) -> Option<&Token> {
        if self.is_at_end() {
            None
        } else {
            Some(&self.input[self.position])
        }
    }

    pub(super) fn eat(
        &mut self, 
        ttype: TokenType, 
        fail_msg: &str
    ) -> Result<Token> {
        if !self.is_at_end() {
            let token = self.input.get_mut(self.position).unwrap();
            let token_type= token.get_type();
            let mut taken = false;
            if token_type == ttype {
                self.position += 1;
                return Ok(token.take());
            }
        }
        Err(Error::Parse(fail_msg.to_string()))
    }

    pub(super) fn eat_current(&mut self) -> Token {
        if self.is_at_end() {
            panic!("Internal error: called eat_current on an empty input");
        }
        let token = self.input.get_mut(self.position).unwrap();
        self.position += 1;
        token.take()
    }

    fn synchronize(&mut self) {
        use TokenType::*;
        let mut token;
        while !self.is_at_end() {
            token = self.eat_current();
            if token.get_type() == Semicolon {
                return;
            }
            if let Some(next_token) = self.peek() {
                if next_token.is_synchronizer() {
                    return;
                }
            } else {
                break;
            }
        }
    }

    fn top_decl(&mut self) -> Result<TopDeclaration> {
        // currently, only function declarations are supported
        let type_token = self.eat_current();
        match type_token.get_type() {
            TokenType::Int => {
                let name_token = self.eat(TokenType::Identifier, "Expected an identifier for function declaration.")?;
                let name_span = name_token.span;
                let name_str = match name_token.inner {
                    RawToken::Identifier(str) => str,
                    _ => return Err(Error::Parse("Expected an identifier for function declaration.".into())),
                };
                
                self.eat(TokenType::LParen, "Expected '(' after function name.")?;
                // currently, only void parameters are supported
                self.eat(TokenType::Void, "Expected 'void' for function parameters.")?;
                self.eat(TokenType::RParen, "Expected ')' after function parameters.")?;
                self.eat(TokenType::LBrace, "Expected '{' to start function body.")?;
                
                let mut body = vec![];
                while !self.is_at_end() && self.peek().unwrap().get_type() != TokenType::RBrace {
                    body.push(self.stmt_top_level()?);
                }

                self.eat(TokenType::RBrace, "Expected '}' to end function body.")?;
                Ok(TopDeclaration::FuncDecl {
                    return_type: (DataType::Int, type_token.span),
                    name: (name_str, name_span),
                    body,
                })
            },
            _ => Err(Error::Parse("Expected a type for function declaration.".into())),
        }
    }
}