//! Location information used when errors are reported in lexing, parsing and resolving stages.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    pub line: usize,
    pub column: usize,
    pub length: Option<usize>,
}

impl Span {
    pub fn new(line: usize, column: usize) -> Self {
        Span { line, column, length: None }
    }

    pub fn with_length(self, length: usize) -> Self {
        let mut span = self;
        span.length = Some(length);
        span
    }
}