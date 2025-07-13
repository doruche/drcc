//! Assembly code emission module
//! LIR -> RISC-V Assembly
//! Here, we do the final assembly code emission from the low-level intermediate representation (LIR).
//! Register allocation, instruction unfolding, and some other hardware-related tasks are performed here.

use std::fmt::Display;

use crate::common::StrDescriptor;

