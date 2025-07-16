use crate::common::*;
use super::{
    TopLevel,
    Decl,
    Expr,
    Stmt,
    BlockItem,
    UnaryOp,
    BinaryOp,
};


#[derive(Debug)]
pub struct Parser {
    input: Vec<Token>,
    position: usize,
    has_error: bool,
    strtb: StringPool,
}

impl Parser {
    pub fn new(input: Vec<Token>, strtb: StringPool) -> Self {
        Self {
            input,
            position: 0,
            has_error: false,
            strtb,
        }
    }

    pub fn strtb(self) -> StringPool {
        self.strtb
    }

    pub fn parse_prog(mut self) -> Result<TopLevel> {
        let mut decls = vec![];
        let mut errors = vec![];
        while !self.is_at_end() {
            match self.decl() {
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
            Ok(TopLevel { decls, strtb: self.strtb })
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

    pub(super) fn peek_next(&self) -> Option<&Token> {
        if self.position + 1 < self.input.len() {
            Some(&self.input[self.position + 1])
        } else {
            None
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
            if token.get_type() == RBrace {
                break;
            }
            if let Some(next_token) = self.peek() {
                if next_token.get_type() == LBrace {
                    break;
                }
            } else {
                break;
            }
        }
    }

    pub(super) fn block_item(&mut self) -> Result<BlockItem> {
        if self.is_at_end() {
            return Err(Error::Parse("Unexpected end of input while parsing block item.".into()));
        }
        match self.peek().unwrap().get_type() {
            TokenType::Int | TokenType::Void => {
                let decl = self.decl()?;
                Ok(BlockItem::Declaration(decl))
            },
            _ => {
                let stmt = self.stmt_top_level()?;
                Ok(BlockItem::Statement(stmt))
            }
        }
    }
}