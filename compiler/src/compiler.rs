use crate::token::{Token, TokenType};
use std::fmt::Debug;
use vm::{FunctionObject, ObjectPtr, RawObject};
#[derive(Debug, Clone, Copy)]
pub struct Local<'a> {
    pub name: Token<'a>,
    pub depth: isize,
}
#[derive(Debug, PartialEq, Eq)]
pub enum FunctionType {
    Function,
    Script,
}

#[derive(Debug, Clone, Copy)]
pub struct UpValue {
    pub index: u8,
    pub is_local: bool,
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
    pub function: ObjectPtr<FunctionObject<'a>>,
    pub compiler_type: FunctionType,
    pub locals: [Local<'a>; 257],
    pub upvalues: [Option<UpValue>; 257],
    pub local_count: usize,
    pub scope_depth: isize,
    pub enclosing: Option<usize>,
}

impl<'a> Compiler<'a> {
    pub fn new(compiler_type: FunctionType, next: RawObject) -> Self {
        Self {
            locals: [Local::default(); 257],
            enclosing: None,
            local_count: 1,
            scope_depth: 0,
            function: FunctionObject::new(None, next),
            compiler_type,
            upvalues: [None; 257],
        }
    }
}

impl<'a> Debug for Compiler<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Compiler")
            .field("function", &self.function)
            .field("compiler_type", &self.compiler_type)
            .field("locals", &&self.locals[0..self.local_count])
            .field("upvalues", &&self.upvalues[0..self.function.upvalue_count])
            .field("local_count", &self.local_count)
            .field("enclosing", &self.enclosing)
            .field("scope_depth", &self.scope_depth)
            .finish()
    }
}
