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