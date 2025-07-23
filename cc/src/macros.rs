//! Some convenient macros

// Create a new span
#[macro_export]
macro_rules! span {
    () => {
        $crate::common::Span::new(0, 0)
    };
    ($line:expr, $column:expr) => {
        $crate::common::Span::new($line, $column)
    };
    ($line:expr, $column:expr, $length:expr) => {
        $crate::common::Span::new($line, $column).with_length($length)
    };
}

#[macro_export]
macro_rules! expect_register {
    ($operand:expr) => {
        if let $crate::lir::LirOperand::PhysReg(inner) = $operand {
            inner
        } else {
            panic!("Internal error: expect a register, found: {:?}", $operand);
        }
    };
}

#[macro_export]
macro_rules! expect_mem {
    ($operand:expr) => {
        if let $crate::lir::LirOperand::Mem { base, offset, size } = $operand {
            (base, offset, size)
        } else {
            panic!("Internal error: expect a memory operand, found: {:?}", $operand);
        }
    };
}