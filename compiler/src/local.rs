use crate::token::{Token, TokenType};
#[derive(Clone, Copy)]
pub struct Local<'a> {
    pub name: Token<'a>,
    pub depth: isize,
}

impl<'a> Default for Local<'a> {
    fn default() -> Self {
        Self {
            name: Token {
                ty: TokenType::Eof,
                lexme: "\0",
                length: 0,
                line: 0,
            },
            depth: Default::default(),
        }
    }
}

pub struct Compiler<'a> {
    pub locals: [Local<'a>; 257],
    pub local_count: usize,
    pub scope_depth: isize,
}

impl<'a> Compiler<'a> {
    pub fn new() -> Self {
        Self {
            locals: [Local::default(); 257],
            local_count: 0,
            scope_depth: 0,
        }
    }
}
