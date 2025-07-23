use crate::common::*;
use super::{
    TopLevel,
    Function,
    StaticVar,
    DataSegment,
    BssSegment,
    Operand,
    LabelOperand,
    IntermediateInsn,
    Insn,
};

impl TopLevel {
    pub fn emit(&self) -> String {
        let mut output = String::new();
        output.push_str(&self.emit_code());
        output.push_str(&self.emit_data());
        output.push_str(&self.emit_bss());
        output
    }

    pub fn emit_data(&self) -> String {
        let mut output = String::new();
        if self.data_seg.items.is_empty() {
            return output;
        }

        output.push_str("\t.data\n");
        for (&name, var) in self.data_seg.items.iter() {
            let name = self.strtb.get(var.name).unwrap();
            if let Linkage::External = var.linkage {
                output.push_str(&format!("\t.globl\t{}\n", name));
            }
            output.push_str(&format!("\t.align\t{}\n", var.data_type.align()));
            output.push_str(&format!("\t.type\t{}, @object\n", name));
            output.push_str(&format!("\t.size\t{}, {}\n", name, var.data_type.size()));
            output.push_str(&format!("{}:\n", name));
            let init_str = match var.initializer {
                InitVal::Const(Constant::Int(i)) =>
                    format!("\t.word\t{}\n", i),
                InitVal::Const(Constant::Long(l)) =>
                    format!("\t.dword\t{}\n", l),
                InitVal::Tentative => unreachable!(), // these should be put in .bss segment
                InitVal::None => unreachable!(),
            };
            output.push_str(&init_str);
        }

        output
    }

    pub fn emit_bss(&self) -> String {
        let mut output = String::new();
        if self.bss_seg.items.is_empty() {
            return output;
        }

        output.push_str("\t.bss\n");
        for (&name, var) in self.bss_seg.items.iter() {
            let name = self.strtb.get(var.name).unwrap();
            if let Linkage::External = var.linkage {
                output.push_str(&format!("\t.globl\t{}\n", name));
            }
            output.push_str(&format!("\t.align\t{}\n", var.data_type.align()));
            output.push_str(&format!("\t.type\t{}, @object\n", name));
            output.push_str(&format!("\t.size\t{}, {}\n", name, var.data_type.size()));
            output.push_str(&format!("{}:\n", name));
            output.push_str(&format!("\t.zero\t{}\n", var.data_type.size()));
        }

        output
    }

    pub fn emit_code(&self) -> String {
        let mut output = String::new();

        output.push_str("\t.text\n");
        for (name, func) in &self.functions {
            output.push_str(&self.emit_func(func));
        }
        output
    }

    pub fn emit_func(&self, func: &Function) -> String {
        let mut output = String::new();
        output.push_str("\t.align\t1\n");

        let name = self.strtb.get(func.name).unwrap();
        if let Linkage::External = func.linkage {
            output.push_str(&format!("\t.global\t{}\n", name));
        }
        output.push_str(&format!("\t.type\t{}, @function\n", name));
        output.push_str(&format!("{}:\n", name));

        for insn in &func.body {
            let prefix = if let Insn::Label(..) = insn {
                "".to_string()
            } else {
                "\t".to_string()
            };
            output.push_str(&format!("{}{}\n", prefix, self.emit_insn(insn)));
        }

        output.push_str(&format!("\t.size\t{}, .-{}\n", name, name));

        output
    }

    pub fn emit_insn(&self, insn: &Insn) -> String {
        use Insn::*;
        
        let mut output = String::new();
        
        match insn {
            Add(rd, rs1, rs2) =>
                output.push_str(&format!("add\t{}, {}, {}", self.emit_operand(rd), self.emit_operand(rs1), self.emit_operand(rs2))),
            Addw(rd, rs1, rs2) =>
                output.push_str(&format!("addw\t{}, {}, {}", self.emit_operand(rd), self.emit_operand(rs1), self.emit_operand(rs2))),
            Sub(rd, rs1, rs2) =>
                output.push_str(&format!("sub\t{}, {}, {}", self.emit_operand(rd), self.emit_operand(rs1), self.emit_operand(rs2))),
            Subw(rd, rs1, rs2) =>
                output.push_str(&format!("subw\t{}, {}, {}", self.emit_operand(rd), self.emit_operand(rs1), self.emit_operand(rs2))),
            Addi(rd, rs1, imm) =>
                output.push_str(&format!("addi\t{}, {}, {}", self.emit_operand(rd), self.emit_operand(rs1), imm)),
            Addiw(rd, rs1, imm) =>
                output.push_str(&format!("addiw\t{}, {}, {}", self.emit_operand(rd), self.emit_operand(rs1), imm)),
            Mul(rd, rs1, rs2) =>
                output.push_str(&format!("mul\t{}, {}, {}", self.emit_operand(rd), self.emit_operand(rs1), self.emit_operand(rs2))),
            Mulw(rd, rs1, rs2) =>
                output.push_str(&format!("mulw\t{}, {}, {}", self.emit_operand(rd), self.emit_operand(rs1), self.emit_operand(rs2))),
            Div(rd, rs1, rs2) =>
                output.push_str(&format!("div\t{}, {}, {}", self.emit_operand(rd), self.emit_operand(rs1), self.emit_operand(rs2))),
            Divw(rd, rs1, rs2) =>
                output.push_str(&format!("divw\t{}, {}, {}", self.emit_operand(rd), self.emit_operand(rs1), self.emit_operand(rs2))),
            Rem(rd, rs1, rs2) =>
                output.push_str(&format!("rem\t{}, {}, {}", self.emit_operand(rd), self.emit_operand(rs1), self.emit_operand(rs2))),
            Remw(rd, rs1, rs2) =>
                output.push_str(&format!("remw\t{}, {}, {}", self.emit_operand(rd), self.emit_operand(rs1), self.emit_operand(rs2))),
            Slt(rd, rs1, rs2) =>
                output.push_str(&format!("slt\t{}, {}, {}", self.emit_operand(rd), self.emit_operand(rs1), self.emit_operand(rs2))),
            Sgt(rd, rs1, rs2) =>
                output.push_str(&format!("sgt\t{}, {}, {}", self.emit_operand(rd), self.emit_operand(rs1), self.emit_operand(rs2))),
            Seqz(rd, rs) =>
                output.push_str(&format!("seqz\t{}, {}", self.emit_operand(rd), self.emit_operand(rs))),
            Snez(rd, rs) =>
                output.push_str(&format!("snez\t{}, {}", self.emit_operand(rd), self.emit_operand(rs))),
            Sextw(rd, rs) =>
                output.push_str(&format!("sext.w\t{}, {}", self.emit_operand(rd), self.emit_operand(rs))),
            Label(label) =>
                output.push_str(&format!("{}:", self.emit_label_operand(label))),
            J(label) =>
                output.push_str(&format!("j\t{}", self.emit_label_operand(label))),
            Beq(rs1, rs2, label) =>
                output.push_str(&format!("beq\t{}, {}, {}", self.emit_operand(rs1), self.emit_operand(rs2), self.emit_label_operand(label))),
            Bne(rs1, rs2, label) =>
                output.push_str(&format!("bne\t{}, {}, {}", self.emit_operand(rs1), self.emit_operand(rs2), self.emit_label_operand(label))),
            Call(name) =>
                output.push_str(&format!("call\t{}", self.strtb.get(*name).unwrap())),
            LoadStatic(rd, name) => 
                output.push_str(&format!("load_static\t{}, {}", self.emit_operand(rd), self.strtb.get(*name).unwrap())),
            StoreStatic(rs, name) =>
                output.push_str(&format!("store_static\t{}, {}", self.emit_operand(rs), self.strtb.get(*name).unwrap())),
            Ret =>
                output.push_str("ret"),
            Lw(rd, mem) =>
                output.push_str(&format!("lw\t{}, {}", self.emit_operand(rd), self.emit_operand(mem))),
            Sw(rs, mem) =>
                output.push_str(&format!("sw\t{}, {}", self.emit_operand(rs), self.emit_operand(mem))),
            Ld(rd, mem) =>
                output.push_str(&format!("ld\t{}, {}", self.emit_operand(rd), self.emit_operand(mem))),
            Sd(rs, mem) =>
                output.push_str(&format!("sd\t{}, {}", self.emit_operand(rs), self.emit_operand(mem))),
            Mv(rd, rs) =>
                output.push_str(&format!("mv\t{}, {}", self.emit_operand(rd), self.emit_operand(rs))),
            Li(rd, imm) =>
                output.push_str(&format!("li\t{}, {}", self.emit_operand(rd), imm)),
            La(rd, name) =>
                output.push_str(&format!("la\t{}, {}", self.emit_operand(rd), self.strtb.get(*name).unwrap())),
            Neg(rd, rs) =>
                output.push_str(&format!("neg\t{}, {}", self.emit_operand(rd), self.emit_operand(rs))),
            Negw(rd, rs) =>
                output.push_str(&format!("negw\t{}, {}", self.emit_operand(rd), self.emit_operand(rs))),
            Not(rd, rs) =>
                output.push_str(&format!("not\t{}, {}", self.emit_operand(rd), self.emit_operand(rs))),
            Intermediate(insn) => match insn {
                IntermediateInsn::Prologue => output.push_str("prologue;"),
                IntermediateInsn::Epilogue => output.push_str("epilogue;"),
            }
        }

        output
    }

    pub fn emit_operand(&self, operand: &Operand) -> String {
        use Operand::*;

        match operand {
            VirtReg(id) => format!("v{}", id),
            PhysReg(reg) => format!("{}", reg),
            Imm(constant) => format!("{}", constant),
            Mem{ base, offset, .. } => format!("{}({})", offset, base),
            Static(name, ..) => format!("{}", self.strtb.get(*name).unwrap()),
        }
    }

    pub fn emit_label_operand(&self, label: &LabelOperand) -> String {
        use LabelOperand::*;

        match label {
            AutoGen(id) => format!(".L{}", id),
            Named(name) => self.strtb.get(*name).unwrap().to_string(),
        }
    }
}