//! Second pass of LIR code generation.
//! Fixes up invalid operands, ensuring all instructions follow the canonical form.
//! e.g.    add t0, a(t1), b(t2) becomes
//!         ld t1, a(t1)
//!         ld t2, b(t2)
//!         add t0, t1, t2

use crate::common::*;
use super::{
    CodeGen,
    Canonic,
    RegAlloc,
    TopLevel,
    Function,
    StaticVar,
    DataSegment,
    BssSegment,
    Insn,
    Operand,
};

impl CodeGen<Canonic> {

}