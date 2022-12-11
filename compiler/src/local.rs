use crate::token::Token;
#[derive(Clone)]
pub struct Local<'a> {
    name: Token<'a>,
    depth: usize,
}

pub struct Compiler<'a> {
    locals: Vec<Local<'a>>,
    pub local_count: usize,
    pub scope_depth: usize,
}

impl<'a> Compiler<'a> {
    pub fn new() -> Self {
        Self {
            locals: Default::default(),
            local_count: 0,
            scope_depth: 0,
        }
    }
}
