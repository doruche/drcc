use crate::common::*;
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
    pub fn emit_static_vars(&self) -> String {
        let mut output = String::new();
        for (name, var) in &self.static_vars {
            let name = self.strtb.get(*name).unwrap();
            let initializer = match &var.initializer {
                InitVal::None => "= undefined".to_string(),
                InitVal::Const(val) => format!("= {}", val),
                InitVal::Tentative => "= tentative".to_string(),
            };
            output.push_str(&format!(
                "[{}]\n{} {} {};\n",
                var.linkage,
                var.data_type,
                name,
                initializer,
            ));
        }
        output
    }

    pub fn emit_code(&self) -> String {
        let mut output = String::new();
        for (name, func) in &self.functions {
            output.push_str(&self.emit_func(func));
        }
        output
    }

    pub(super) fn emit_func(&self, func: &Function) -> String {
        let mut output = String::new();
        let params = func.params.iter()
            .map(|param| format!(
                "{} {}",
                param.data_type,
                self.strtb.get(param.name).unwrap(),
            ))
            .collect::<Vec<_>>()
            .join(", ");
        let signature = format!(
            "[{}]\nfn {} ({}) -> {}",
            func.linkage,
            self.strtb.get(func.name).unwrap(),
            if params.is_empty() { "void".to_string() } else { params },
            func.return_type,
        );
        output.push_str(&signature);
        output.push('\n');
        output.push_str("code:\n");
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

        output.push_str("local vars:\n");
        for (_, var) in &func.local_vars {
            let name = self.strtb.get(var.name).unwrap();
            output.push_str(&format!(
                "\t{} %{}.{};\n",
                var.data_type,
                name,
                var.local_id,
            ));
        }

        output.push('\n');

        output
    }

    fn emit_insn(&self, insn: &Insn) -> String {
        match insn {
            Insn::Return(val) => {
                let val_str = self.emit_operand(val);
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
                    BinaryOp::And => unreachable!(),
                    BinaryOp::Or => unreachable!(),
                }
            },
            Insn::FuncCall { target, args, dst } => {
                let postfix = if args.is_empty() {
                    "".to_string()
                } else {
                    let args_str = args.iter()
                        .map(|arg| self.emit_operand(arg))
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!("with ({})", args_str)
                };
                format!("call\t{}, {} {}", 
                    self.strtb.get(*target).unwrap(), self.emit_operand(dst), postfix)
            }
            Insn::BranchIfZero { src, label }
                => format!("bz\t{}, {}", self.emit_operand(src), self.emit_label_operand(label)),
            Insn::BranchNotZero { src, label }
                => format!("bnz\t{}, {}", self.emit_operand(src), self.emit_label_operand(label)),
            Insn::Label(label) => format!("{}:", self.emit_label_operand(label)),
            Insn::Jump(label) => format!("jmp\t{}", self.emit_label_operand(label)),
            Insn::Move { src, dst} => 
                format!("mov\t{}, {}", self.emit_operand(dst), self.emit_operand(src)),
            Insn::Truncate { src, dst } => 
                format!("trunc\t{}, {}", self.emit_operand(dst), self.emit_operand(src)),
            Insn::SignExt { src, dst } => 
                format!("sext\t{}, {}", self.emit_operand(dst), self.emit_operand(src)),
        }
    }

    fn emit_operand(&self, operand: &Operand) -> String {
        match operand {
            Operand::Imm(imm) => match imm {
                Constant::Int(val) => format!("$[i32]{}", val),
                Constant::Long(val) => format!("$[i64]{}", val),
            },
            Operand::Temp(tid, data_type) => format!("%[{}]t.{}", data_type.sized_str(), tid),
            Operand::Var {
                name,
                local_id,
                data_type,
            } => {
                let name = self.strtb.get(*name).unwrap();
                match local_id {
                    Some(id) => format!("%[{}]{}.{}", data_type.sized_str(), name, id),
                    None => format!("%[{}]{}", data_type.sized_str(), name),
                }
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

