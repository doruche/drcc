use std::fmt::Display;
use super::{
    Operand,
    Insn,
    Function,
    TopLevel,
    UnaryOp,
    BinaryOp,
};


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
            Insn::Return(Some(operand)) => write!(f, "ret {}", operand),
            Insn::Return(None) => write!(f, "ret"),
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
                BinaryOp::Ls => write!(f, "lt {}, {}, {}", dst, left, right),
                BinaryOp::Gt => write!(f, "gt {}, {}, {}", dst, left, right),
                BinaryOp::GtEq => write!(f, "gte {}, {}, {}", dst, left, right),
                BinaryOp::LsEq => write!(f, "lte {}, {}, {}", dst, left, right),
                BinaryOp::Eq => write!(f, "eq {}, {}, {}", dst, left, right),
                BinaryOp::NotEq => write!(f, "ne {}, {}, {}", dst, left, right),
                BinaryOp::And|BinaryOp::Or => unreachable!("Combined by other insns"),
                BinaryOp::Assign => todo!(),
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

