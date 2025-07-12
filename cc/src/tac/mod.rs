//! Three-Address Code (TAC) module.
//! AST -> TAC

use std::fmt::Display;

use crate::common::{BinaryOp, DataType, StrDescriptor, StringPool, UnaryOp};

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
    },
    Label(usize),
    Jump(usize),
    BranchIfZero {
        src: Operand,
        label: usize,
    },
    BranchNotZero {
        src: Operand,
        label: usize,
    },
    Move {
        src: Operand,
        dst: Operand,
    },
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
    fn dump(&self, strtb: &StringPool) -> String {
        let insns = format!("{:#}", self);
        let strtb = strtb.dump();
        format!("{}\nString Table:\n{}", insns, strtb)
    }
}

impl Display for Operand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operand::Imm(value) => write!(f, "{}", value),
            Operand::Var(name) => write!(f, "[{}]", name.index()),
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
                UnaryOp::Complement => write!(f, "cpl {}, {}", dst, src),
                UnaryOp::Negate => write!(f, "neg {}, {}", dst, src),
                UnaryOp::Pos => write!(f, "pos {}, {}", dst, src),
                UnaryOp::Not => write!(f, "not {}, {}", dst, src),
            },
            Insn::Binary { op, left, right, dst } => match op {
                BinaryOp::Add => write!(f, "add {}, {}, {}", dst, left, right),
                BinaryOp::Sub => write!(f, "sub {}, {}, {}", dst, left, right),
                BinaryOp::Mul => write!(f, "mul {}, {}, {}", dst, left, right),
                BinaryOp::Div => write!(f, "div {}, {}, {}", dst, left, right),
                BinaryOp::Rem => write!(f, "rem {}, {}, {}", dst, left, right),
                BinaryOp::LessThan => write!(f, "lt {}, {}, {}", dst, left, right),
                BinaryOp::GreaterThan => write!(f, "gt {}, {}, {}", dst, left, right),
                BinaryOp::GtEq => write!(f, "gte {}, {}, {}", dst, left, right),
                BinaryOp::LtEq => write!(f, "lte {}, {}, {}", dst, left, right),
                BinaryOp::Equal => write!(f, "eq {}, {}, {}", dst, left, right),
                BinaryOp::NotEqual => write!(f, "ne {}, {}, {}", dst, left, right),
                BinaryOp::And|BinaryOp::Or => unreachable!("Combined by other insns"),
            },
            Insn::Label(name) => write!(f, "\rL.{}:", name),
            Insn::Jump(label) => write!(f, "jmp L.{}", label),
            Insn::BranchIfZero { src, label } => write!(f, "biz {}, L.{}", src, label),
            Insn::BranchNotZero { src, label } => write!(f, "bnz {}, L.{}", src, label),
            Insn::Move { src, dst } => write!(f, "mov {}, {}", dst, src),
        }
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "fn [{}]: {:?} {{\n", self.name.index(), self.return_type)?;
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
    use std::fs::read_to_string;

    use crate::{ast::{AstParser, Expr}, lex::Lexer, span};

    use super::*;

    #[test]
    fn test_expr() {
        let mut lexer = Lexer::new("1<=3*7>5==1".into());
        let (tokens, _pool) = lexer.lex().unwrap();
        let mut parser = AstParser::new(tokens);
        let expr = parser.parse_expr().unwrap();
        println!("{:#?}", expr);

        let result = codegen::parse_expr(expr, &mut 0, &mut 0);
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
        // let input = "int main(void) { return 1 * 2 - 3 * (4 + 5); }";
        let input = read_to_string("../testprogs/basic.c").unwrap();
        let mut lexer = Lexer::new(input.into());
        let (tokens, pool) = lexer.lex().unwrap();
        let mut parser = AstParser::new(tokens);
        let prog = parser.parse_prog().unwrap();
        let mut parser = TacParser::new(prog);
        let result = parser.parse();
        match result {
            Ok(tac) => {
                println!("{:#}", tac.dump(&pool));
            }
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
    }
}