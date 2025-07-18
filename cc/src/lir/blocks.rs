use crate::common::*;
use super::{
    codegen,
    TopLevel,
    StaticVar,
    Function,
    Operand,
    Insn,
    DataSegment,
    BssSegment,
};

impl Function {

}

impl DataSegment {
    pub fn new() -> Self {
        DataSegment { items: vec![] }
    }

    pub fn add(&mut self, var: StaticVar) {
        self.items.push(var);
    }
}

impl BssSegment {
    pub fn new() -> Self {
        BssSegment { items: vec![] }
    }

    pub fn add(&mut self, var: StaticVar) {
        self.items.push(var);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::*;
}