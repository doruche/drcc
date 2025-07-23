use crate::common::*;
use super::{
    Insn,
    CodeGen,
    Register,
    LabelOperand,
    TopLevel,
    Function,
    StaticVar,
    DataSegment,
    BssSegment,
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
        output.push('\n');

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
        output.push('\n');

        output
    }

    pub fn emit_code(&self) -> String {
        let mut output = String::new();
        
        output.push_str("\t.text\n");
        for func in self.functions.values() {
            output.push_str(&self.emit_func(func));
        }

        output
    }

    pub fn emit_func(&self, func: &Function) -> String {
        let mut output = String::new();
        output.push_str("\t.align\t1\n");

        let name = self.strtb.get(func.name).unwrap();
        if let Linkage::External = func.linkage {
            output.push_str(&format!("\t.globl\t{}\n", name));
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

        output.push_str(&format!("\t.size\t{}, .-{}\n\n", name, name));

        output
    }

    pub fn emit_insn(&self, insn: &Insn) -> String {
        use Insn::*;
        let mut output = String::new();

        match insn {
            Add(rd, rs1, rs2) =>
                output.push_str(&format!("add\t{}, {}, {}", rd, rs1, rs2)),
            Addw(rd, rs1, rs2) =>
                output.push_str(&format!("addw\t{}, {}, {}", rd, rs1, rs2)),
            Addi(rd, rs1, imm) =>
                output.push_str(&format!("addi\t{}, {}, {}", rd, rs1, imm)),
            Addiw(rd, rs1, imm) =>
                output.push_str(&format!("addiw\t{}, {}, {}", rd, rs1, imm)),
            Sub(rd, rs1, rs2) =>
                output.push_str(&format!("sub\t{}, {}, {}", rd, rs1, rs2)),
            Subw(rd, rs1, rs2) =>
                output.push_str(&format!("subw\t{}, {}, {}", rd, rs1, rs2)),
            Mul(rd, rs1, rs2) =>
                output.push_str(&format!("mul\t{}, {}, {}", rd, rs1, rs2)),
            Mulw(rd, rs1, rs2) =>
                output.push_str(&format!("mulw\t{}, {}, {}", rd, rs1, rs2)),
            Div(rd, rs1, rs2) =>
                output.push_str(&format!("div\t{}, {}, {}", rd, rs1, rs2)),
            Divw(rd, rs1, rs2) =>
                output.push_str(&format!("divw\t{}, {}, {}", rd, rs1, rs2)),
            Rem(rd, rs1, rs2) =>
                output.push_str(&format!("rem\t{}, {}, {}", rd, rs1, rs2)),
            Remw(rd, rs1, rs2) =>
                output.push_str(&format!("remw\t{}, {}, {}", rd, rs1, rs2)),
            Slt(rd, rs1, rs2) =>
                output.push_str(&format!("slt\t{}, {}, {}", rd, rs1, rs2)),
            Sgt(rd, rs1, rs2) =>
                output.push_str(&format!("sgt\t{}, {}, {}", rd, rs1, rs2)),
            Seqz(rd, rs) =>
                output.push_str(&format!("seqz\t{}, {}", rd, rs)),
            Snez(rd, rs) =>
                output.push_str(&format!("snez\t{}, {}", rd, rs)),
            Sextw(rd, rs) =>
                output.push_str(&format!("sext.w\t{}, {}", rd, rs)),
            Mv(rd, rs) =>
                output.push_str(&format!("mv\t{}, {}", rd, rs)),
            Neg(rd, rs) =>
                output.push_str(&format!("neg\t{}, {}", rd, rs)),
            Negw(rd, rs) =>
                output.push_str(&format!("negw\t{}, {}", rd, rs)),
            Not(rd, rs) =>
                output.push_str(&format!("not\t{}, {}", rd, rs)),
            Call(name) =>
                output.push_str(&format!("call\t{}", self.strtb.get(*name).unwrap())),
            Beq(rs1, rs2, label) =>
                output.push_str(&format!("beq\t{}, {}, {}", rs1, rs2, self.emit_label_operand(label))),
            Bne(rs1, rs2, label) =>
                output.push_str(&format!("bne\t{}, {}, {}", rs1, rs2, self.emit_label_operand(label))),
            J(label) =>
                output.push_str(&format!("j\t{}", self.emit_label_operand(label))),
            Label(label) =>
                output.push_str(&format!("{}:", self.emit_label_operand(label))),
            Ret => output.push_str("ret"),
            Ld(rd, base, offset) =>
                output.push_str(&format!("ld\t{}, {}({})", rd, offset, base)),
            Lw(rd, base, offset) =>
                output.push_str(&format!("lw\t{}, {}({})", rd, offset, base)),
            Sd(rs, base, offset) =>
                output.push_str(&format!("sd\t{}, {}({})", rs, offset, base)),
            Sw(rs, base, offset) =>
                output.push_str(&format!("sw\t{}, {}({})", rs, offset, base)),
            Li(rd, imm) =>
                output.push_str(&format!("li\t{}, {}", rd, imm)),
            La(rd, name) =>
                output.push_str(&format!("la\t{}, {}", rd, self.strtb.get(*name).unwrap())),
            LoadStatic(rd, name) => {
                let static_var = self.get_static_var(name).expect("Static variable not found");
                let name = self.strtb.get(static_var.name).unwrap();
                output.push_str(&format!("lui\tt5, %hi({})\n", name));
                match static_var.data_type.size() {
                    4 => output.push_str(&format!("\tlw\t{}, %lo({})(t5)\n", rd, name)),
                    8 => output.push_str(&format!("\tld\t{}, %lo({})(t5)\n", rd, name)),
                    _ => unreachable!(),
                }
            },
            StoreStatic(rs, name) => {
                let static_var = self.get_static_var(name).expect("Static variable not found");
                let name = self.strtb.get(static_var.name).unwrap();
                output.push_str(&format!("lui\tt5, %hi({})\n", name));
                match static_var.data_type.size() {
                    4 => output.push_str(&format!("\tsw\t{}, %lo({})(t5)\n", rs, name)),
                    8 => output.push_str(&format!("\tsd\t{}, %lo({})(t5)\n", rs, name)),
                    _ => unreachable!(),
                }
            },
        }
        output
    }

    pub fn emit_label_operand(&self, label: &LabelOperand) -> String {
        match label {
            LabelOperand::AutoGen(id) => format!(".L{}", id),
            _ => todo!()
        }
    }
}

impl TopLevel {
    fn get_static_var(&self, name: &StrDescriptor) -> Option<&StaticVar> {
        self.data_seg.items.get(name).or_else(|| self.bss_seg.items.get(name))
    }
}