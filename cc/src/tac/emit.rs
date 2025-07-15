use std::fmt::Display;
use super::{
    Operand,
    Insn,
    Function,
    TopLevel,
    UnaryOp,
    BinaryOp,
    LabelOperand,
    AutoGenLabel,
};

impl TopLevel {
    pub fn emit(&self) -> String {
        let mut output = String::new();
        for func in &self.functions {
            output.push_str(&self.emit_func(func));
        }
        output
    }

    fn emit_func(&self, func: &Function) -> String {
        let mut output = String::new();
        let signature = format!(
            "fn {} (void) -> int",
            self.strtb.get(func.name).unwrap(),
        );
        output.push_str(&signature);
        output.push('\n');
        for insn in &func.body {
            let prefix = if let Insn::Label(..) = insn {
                "".to_string()
            } else {
                "\t".to_string()
            };
            let insn_str = self.emit_insn(insn);
            output.push_str(&format!("{}{}\n", prefix, insn_str));
        }
        output.push_str("\n");

        output
    }

    fn emit_insn(&self, insn: &Insn) -> String {
        match insn {
            Insn::Return(val) => {
                let val_str = if let Some(operand) = val {
                    self.emit_operand(operand)
                } else {
                    "".to_string()
                };
                format!("ret {}", val_str)
            },
            Insn::Unary {
                op,
                src,
                dst,
            } => {
                match op {
                    UnaryOp::Pos => "".to_string(),
                    UnaryOp::Negate =>
                        format!("neg\t{}, {}", self.emit_operand(dst), self.emit_operand(src)),
                    UnaryOp::Not =>
                        format!("not\t{}, {}", self.emit_operand(dst), self.emit_operand(src)),
                    UnaryOp::Complement =>
                        format!("cmpl\t{}, {}", self.emit_operand(dst), self.emit_operand(src)),
                }
            },
            Insn::Binary {
                op,
                left,
                right,
                dst,
            } => {
                let left_str = self.emit_operand(left);
                let right_str = self.emit_operand(right);
                let dst_str = self.emit_operand(dst);
                match op {
                    BinaryOp::Add => format!("add\t{}, {}, {}", dst_str, left_str, right_str),
                    BinaryOp::Sub => format!("sub\t{}, {}, {}", dst_str, left_str, right_str),
                    BinaryOp::Mul => format!("mul\t{}, {}, {}", dst_str, left_str, right_str),
                    BinaryOp::Div => format!("div\t{}, {}, {}", dst_str, left_str, right_str),
                    BinaryOp::Rem => format!("rem\t{}, {}, {}", dst_str, left_str, right_str),
                    BinaryOp::Ls => format!("ls\t{}, {}, {}", dst_str, left_str, right_str),
                    BinaryOp::Gt => format!("gt\t{}, {}, {}", dst_str, left_str, right_str),
                    BinaryOp::GtEq => format!("gte\t{}, {}, {}", dst_str, left_str, right_str),
                    BinaryOp::LsEq => format!("lte\t{}, {}, {}", dst_str, left_str, right_str),
                    BinaryOp::Eq => format!("eq\t{}, {}, {}", dst_str, left_str, right_str),
                    BinaryOp::NotEq => format!("neq\t{}, {}, {}", dst_str, left_str, right_str),
                    BinaryOp::And => format!("and\t{}, {}, {}", dst_str, left_str, right_str),
                    BinaryOp::Or => format!("or\t{}, {}, {}", dst_str, left_str, right_str),
                    BinaryOp::Assign => unreachable!(),
                }
            },
            Insn::BranchIfZero { src, label }
                => format!("bz\t{}, {}", self.emit_operand(src), self.emit_label_operand(label)),
            Insn::BranchNotZero { src, label }
                => format!("bnz\t{}, {}", self.emit_operand(src), self.emit_label_operand(label)),
            Insn::Label(label) => format!("{}:", self.emit_label_operand(label)),
            Insn::Jump(label) => format!("jmp\t{}", self.emit_label_operand(label)),
            Insn::Move { src, dst} => 
                format!("mov\t{}, {}", self.emit_operand(dst), self.emit_operand(src)),
        }
    }

    fn emit_operand(&self, operand: &Operand) -> String {
        match operand {
            Operand::Imm(imm) => imm.to_string(),
            Operand::Temp(tid) => format!("t.{}", tid),
            Operand::Var(sd) => {
                let name = self.strtb.get(*sd).unwrap();
                format!("{}", name)
            },
        }
    }

    fn emit_label_operand(&self, label: &LabelOperand) -> String {
        match label {
            LabelOperand::AutoGen(autogen) => match autogen {
                AutoGenLabel::Branch(id) => format!("bra.{}", id),
                AutoGenLabel::Continue(id) => format!("con.{}", id),
                AutoGenLabel::Break(id) => format!("brk.{}", id),
            },
            LabelOperand::Named { name, id } => {
                let name_str = self.strtb.get(*name).unwrap();
                format!("{}.{}", name_str, id)
            }
        }
    }
}

