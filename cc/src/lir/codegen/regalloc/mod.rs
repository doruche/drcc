//! Final pass of LIR code generation.
//! Map virtual registers to physical registers or spill them to memory.
//! Currently we use a ultra-simple algorithm that just spill all virtual registers
//! to memory, but this will be replaced with a more sophisticated algorithm in the future.

mod spill;