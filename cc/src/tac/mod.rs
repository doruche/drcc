//! Three-Address Code (TAC) module.
//! AST -> TAC

use std::fmt::Display;

use crate::common::{BinaryOp, DataType, StrDescriptor, UnaryOp};

mod codegen;

pub use codegen::Parser as TacParser;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operand {
    Imm(i64),
    Var(StrDescriptor),
    Temp(usize),
}

#[derive(Debug, Clone)]
pub enum Insn {
    Return(Option<Operand>),
    Unary {
        op: UnaryOp,
        src: Operand,
        dst: Operand,
    },
    Binary {
        op: BinaryOp,
        left: Operand,
        right: Operand,
        dst: Operand,
    }
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: StrDescriptor,
    // pub params,
    pub return_type: DataType,
    pub body: Vec<Insn>,
}

#[derive(Debug, Clone)]
pub struct TopLevel {
    pub functions: Vec<Function>,
}

impl TopLevel {
    fn dump(&self) -> String {
        format!("{:#}", self)
    }
}

impl Display for Operand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operand::Imm(value) => write!(f, "{}", value),
            Operand::Var(name) => write!(f, "{}", name.index()),
            Operand::Temp(id) => write!(f, "t{}", id),
        }
    }
}

impl Display for Insn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Insn::Return(Some(operand)) => write!(f, "return {}", operand),
            Insn::Return(None) => write!(f, "return"),
            Insn::Unary { op, src, dst } => match op {
                UnaryOp::Complement => write!(f, "not {}, {}", dst, src),
                UnaryOp::Negate => write!(f, "neg {}, {}", dst, src),
                UnaryOp::Pos => write!(f, "pos {}, {}", dst, src),
            },
            Insn::Binary { op, left, right, dst } => match op {
                BinaryOp::Add => write!(f, "add {}, {}, {}", dst, left, right),
                BinaryOp::Sub => write!(f, "sub {}, {}, {}", dst, left, right),
                BinaryOp::Mul => write!(f, "mul {}, {}, {}", dst, left, right),
                BinaryOp::Div => write!(f, "div {}, {}, {}", dst, left, right),
                BinaryOp::Rem => write!(f, "rem {}, {}, {}", dst, left, right),
            }
        }
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "fn {}: {:?} {{\n", self.name.index(), self.return_type)?;
        for insn in &self.body {
            write!(f, "    {}\n", insn)?;
        }
        write!(f, "}}\n")?;
        Ok(())
    }
}

impl Display for TopLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for function in &self.functions {
            write!(f, "{}", function)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{ast::{AstParser, Expr}, lex::Lexer, span};

    use super::*;

    #[test]
    fn test_expr() {
        let mut lexer = Lexer::new("~+-+42".into());
        let (tokens, _pool) = lexer.lex().unwrap();
        let mut parser = AstParser::new(tokens);
        let expr = parser.parse_expr().unwrap();
        println!("{:#?}", expr);

        let result = codegen::parse_expr(expr, &mut 0);
        match result {
            Ok((operand, insns)) => {
                println!("Operand: {:?}", operand);
                if let Some(insns) = insns {
                    for insn in insns {
                        println!("Insn: {:?}", insn);
                    }
                }
            }
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
    }

    #[test]
    fn test_prog() {
        let input = "int main(void) { return 1 * 2 - 3 * (4 + 5); }";
        let mut lexer = Lexer::new(input.into());
        let (tokens, _pool) = lexer.lex().unwrap();
        let mut parser = AstParser::new(tokens);
        let prog = parser.parse_prog().unwrap();
        let mut parser = TacParser::new(prog);
        let result = parser.parse();
        match result {
            Ok(tac) => {
                println!("{:#}", tac);
            }
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
    }
}