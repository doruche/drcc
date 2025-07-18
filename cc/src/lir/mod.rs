//! Low-Level Intermediate Representation (LIR) module
//! TAC -> LIR
//! In this module, we will:
//! 1. Convert the three-address code (TAC) into a low-level intermediate representation (LIR).
//! 2. Register allocation.
//! 3. Instruction canonicalization.

mod lir;
mod blocks;
mod codegen;

use lir::{
    Operand,
    Insn,
    Function,
    StaticVar,
    TopLevel,
    DataSegment,
    BssSegment,
};

pub use lir::{
    Operand as LirOperand,
    Insn as LirInsn,
    Function as LirFunction,
    StaticVar as LirStaticVar,
    TopLevel as LirTopLevel,
    DataSegment as LirDataSegment,
    BssSegment as LirBssSegment,
};