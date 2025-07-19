use crate::common::*;
use crate::tac::{
    TacTopLevel,
    TacStaticVar,
    TacFunction,
    TacUnaryOp,
    TacBinaryOp,
    AutoGenLabel,
};
use super::{
    TopLevel,
    StaticVar,
    Function,
    Operand,
    Insn,
    DataSegment,
    BssSegment,
};

#[derive(Debug)]
pub struct Parser {

}

impl Parser {
    pub fn new() -> Self {
        Parser {}
    }

    pub fn parse(mut self, tac: TacTopLevel) -> Result<TopLevel> {
        todo!()
    }
}