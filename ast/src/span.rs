#[derive(Debug, Clone)]
pub struct Spanned<T> {
    value: T,
    span: Span,
}

impl<T> Spanned<T> {
    pub fn new(value: T, span: Span) -> Self {
        Spanned { span, value }
    }
}

/// A span between two locations in a source file
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Span {
    pub start: Position,
    pub end: Position,
}

#[derive(Debug, Copy, PartialOrd, Clone, PartialEq, Eq, Ord)]
pub struct Position {
    pub line: u32,
    pub column: u32,
    pub absolute: usize,
}
