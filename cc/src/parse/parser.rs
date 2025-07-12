use crate::{common::{Error, Result, Token, TokenType}, parse::{Expr, Stmt}};

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

    pub fn parse_prog(&mut self) -> Result<Stmt> {
        let mut prog = vec![];
        let mut errors = vec![];
        while !self.is_at_end() {
            match self.declaration() {
                Ok(stmt) => prog.push(stmt),
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
            Ok(Stmt::Program(prog))
        }
    }

    pub fn parse_expr(&mut self) -> Result<Expr> {
        self.primary()
    }

    fn is_at_end(&self) -> bool {
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
}