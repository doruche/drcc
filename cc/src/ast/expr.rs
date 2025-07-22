use crate::{ast::parser::parse_types, common::*};
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
    pub(super) fn expr_top_level(&mut self) -> Result<Expr> {
        self.expr(0)
    }

    fn expr(&mut self, min_prec: u8) -> Result<Expr> {
        let mut left = self.unary()?;
        let mut right;
        loop {
            let next_op_token = self.peek();
            if next_op_token.is_err() || !next_op_token.as_ref().unwrap().is_binary_op() {
                break;
            }
            if next_op_token.unwrap().to_binary_op().precedence() < min_prec {
                break;
            }
            let next_op_token = self.eat_current();
            let next_op = next_op_token.to_binary_op();

            if let BinaryOp::Assign = next_op {
                // right-associative
                right = self.expr(BinaryOp::Assign.precedence())?;
                left = Expr::Assignment {
                    span: next_op_token.span,
                    left: Box::new(left),
                    right: Box::new(right),
                };
            } else if let BinaryOp::Ternary = next_op {
                let middle = self.expr(BinaryOp::MIN_PRECEDENCE)?;
                self.eat(TokenType::Colon, "Expected ':' in ternary expression.")?;
                right = self.expr(BinaryOp::Ternary.precedence())?;
                left = Expr::Ternary {
                    span: next_op_token.span,
                    condition: Box::new(left),
                    then_expr: Box::new(middle),
                    else_expr: Box::new(right),
                };
            } else {
                right = self.expr(next_op.precedence() + 1)?;
                left = Expr::Binary {
                    op: (next_op, next_op_token.span),
                    left: Box::new(left),
                    right: Box::new(right),
                }
            }
        }
        
        Ok(left)
    }

    fn unary(&mut self) -> Result<Expr> {
        if matches!(self.peek()?.get_type(),
        TokenType::Hyphen|TokenType::Plus|TokenType::Tilde|TokenType::Bang) {
            let op_token = self.eat_current();
            let op = match op_token.get_type() {
                TokenType::Hyphen => UnaryOp::Negate,
                TokenType::Plus => UnaryOp::Pos,
                TokenType::Tilde => UnaryOp::Complement,
                TokenType::Bang => UnaryOp::Not,
                _ => unreachable!(),
            };
            let span = op_token.span;
            let expr = self.unary()?;
            Ok(Expr::Unary((op, span), Box::new(expr)))
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> Result<Expr> {
        let token = self.eat_current();
        match token.get_type() {
            TokenType::IntLiteral|TokenType::LongLiteral => {
                let constant = token.inner.as_constant();
                Ok(Expr::IntegerLiteral(constant))
            },
            TokenType::LParen => {
                if self.peek()?.is_type() {
                    let mut types = vec![];
                    loop {
                        if self.peek()?.is_type() {
                            types.push(self.eat_current().inner.as_type());
                        } else if self.peek()?.get_type() == TokenType::RParen {
                            self.eat_current();
                            break;
                        } else {
                            Err(Error::parse("Expected a type or ')'", self.cur_span()))?;
                        }
                    }
                    let target = parse_types(types, self.cur_span())?;
                    let expr = Box::new(self.expr_top_level()?);
                    Ok(Expr::Cast {
                        target,
                        expr,
                        span: token.span,
                    })
                } else {
                    // grouping
                    let expr = self.expr_top_level()?;
                    self.eat(TokenType::RParen, "Expected ')' to close expression.")?;
                    Ok(Expr::Group(Box::new(expr)))
                }
            },
            TokenType::Identifier => {
                let sd = token.inner.as_identifier();
                let span = token.span;

                if let Ok(TokenType::LParen) = self.peek().map(Token::get_type) {
                    self.eat_current();
                    let mut args = vec![];
                    loop {
                        if self.peek()?.get_type() == TokenType::RParen {
                            self.eat_current();
                            break;
                        }
                        args.push(self.expr_top_level()?);
                        match self.peek()?.get_type() {
                            TokenType::Comma => {
                                self.eat_current();
                            },
                            TokenType::RParen => {
                                // do nothing, we already checked for it
                            },
                            _ => Err(Error::parse("Expected ',' or ')' in function call arguments.", span))?
                        }
                    }
                    Ok(Expr::FuncCall {
                        name: sd,
                        span,
                        args,
                    })
                } else {
                    Ok(Expr::Variable(sd, span))
                }
            }
            _ => Err(Error::parse("Expected an expression", token.span)),
        }
    }
}