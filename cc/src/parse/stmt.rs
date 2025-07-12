use crate::{common::{DataType, Error, RawToken, Result, Span, Token, TokenType}, parse::{Expr, Parser}};

#[derive(Debug, Clone)]
pub enum Stmt {
    Program(Vec<Stmt>),
    FuncDecl {
        return_type: (DataType, Span),
        name: (String, Span),
        // params,
        body: Box<Stmt>, // currently, only one statement in the body is supported
    },
    Return {
        span: Span,
        expr: Box<Expr>,
    },
}

impl Parser {
    pub(super) fn declaration(&mut self) -> Result<Stmt> {
        // currently, only "int <identifier>(void) { ... }" is supported
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
                let body = self.statement()?;
                self.eat(TokenType::RBrace, "Expected '}' to end function body.")?;
                Ok(Stmt::FuncDecl {
                    return_type: (DataType::Int, type_token.span),
                    name: (name_str, name_span),
                    body: Box::new(body),
                })
            },
            _ => Err(Error::Parse("Expected a type for function declaration.".into())),
        }
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