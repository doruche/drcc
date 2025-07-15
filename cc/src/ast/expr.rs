use crate::common::*;
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
            if next_op_token.is_none() || !next_op_token.unwrap().is_binary_op() {
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
        if self.is_at_end() {
            return Err(Error::Parse("Unexpected end of input while parsing expression.".into()));
        }
        if matches!(self.peek().unwrap().get_type(),
        TokenType::Hyphen|TokenType::Plus|TokenType::Tilde) {
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
            TokenType::Integer => Ok(Expr::IntegerLiteral(token.inner.as_integer())),
            TokenType::LParen => {
                let expr = self.parse_expr()?;
                self.eat(TokenType::RParen, "Expected ')' to close expression.")?;
                Ok(Expr::Group(Box::new(expr)))
            },
            TokenType::Identifier => {
                // currently only variable
                let sd = token.inner.as_identifier();
                let span = token.span;
                Ok(Expr::Variable(sd, span))
            }
            _ => Err(Error::parse("Expected an expression", token.span)),
        }
    }
}