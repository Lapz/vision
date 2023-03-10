use std::{
    borrow::{Borrow, BorrowMut},
    cmp,
    hash::Hash,
    ops::{Deref, DerefMut},
};

#[derive(Debug, Clone)]
pub struct Spanned<T> {
    value: T,
    span: Span,
}

impl<T> Spanned<T> {
    pub fn new(value: T, span: Span) -> Self {
        Spanned { span, value }
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn into_value(self) -> T {
        self.value
    }

    pub fn merge_span(self, other: &Self) -> Self {
        Spanned {
            value: self.value,
            span: Span::new(self.span.start, other.span.end),
        }
    }

    pub fn extend(&mut self, other: Span) {
        self.span.end = cmp::max(self.span.end, other.end)
    }

    pub fn value(&self) -> &T {
        &self.value
    }

    pub fn view<'a>(&self, src: &'a str) -> Option<&'a str> {
        src.get(self.span.start.absolute..self.span.end.absolute)
    }
}

impl<T> Deref for Spanned<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> DerefMut for Spanned<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.value
    }
}

impl<T> Borrow<T> for Spanned<T> {
    fn borrow(&self) -> &T {
        &self.value
    }
}

impl<T> BorrowMut<T> for Spanned<T> {
    fn borrow_mut(&mut self) -> &mut T {
        &mut self.value
    }
}

impl<T: Hash> Hash for Spanned<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.value.hash(state);
        self.span.hash(state);
    }
}

impl<T: PartialEq> PartialEq for Spanned<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value && self.span == other.span
    }
}

impl<T: Eq> Eq for Spanned<T> {}

impl<T: Copy> Copy for Spanned<T> {}

/// A span between two locations in a source file
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Hash, Eq)]
pub struct Span {
    pub start: Position,
    pub end: Position,
}

impl Span {
    pub fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }

    pub fn merge(&self, other: Span) -> Span {
        Self {
            start: cmp::min(self.start, other.start),
            end: cmp::max(self.end, other.end),
        }
    }
}
#[derive(Debug, Copy, PartialOrd, Clone, PartialEq, Eq, Ord, Hash)]
pub struct Position {
    pub line: u32,
    pub column: u32,
    pub absolute: usize,
}

impl Position {
    pub const fn new(line: u32, column: u32, absolute: usize) -> Self {
        Self {
            line,
            column,
            absolute,
        }
    }

    pub fn shift(mut self, ch: &str) -> Self {
        if ch == "\n" {
            self.line += 1;
            self.column = 1;
        } else if ch == "\t" {
            self.column += 4;
        } else if ch == "\r" {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }

        self.absolute += ch.len();
        self
    }

    pub fn shift_byte(mut self, ch: u8) -> Self {
        if ch == b'\n' {
            self.line += 1;
            self.column = 1;
        } else if ch == b'\t' {
            self.column += 4;
        } else {
            self.column += 1;
        }

        self.absolute += (ch as char).len_utf8();
        self
    }
}
