//! Assembly code emission module
//! LIR -> RISC-V Assembly
//! Here, we do the final assembly code emission from the low-level intermediate representation (LIR).
//! Here we'll do final emission, as well as some hardware-related optimizations. (e.g. peephole optimizations)

mod riscv;

pub use riscv::{
    Register,
};