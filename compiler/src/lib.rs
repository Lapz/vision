mod compiler;
mod parser;
mod scanner;
mod token;
pub mod v2;

use scanner::Scanner;
use vm::{Allocator, FunctionObject, ObjectPtr, Table};

use crate::token::TokenType;

pub fn compile(input: &str) -> Option<ParseResult> {
    let scanner = Scanner::new(input);
    let mut parser = parser::Parser::new(scanner);

    parser.advance();

    while !parser.match_token(TokenType::Eof) {
        parser.declaration();
    }

    if parser.had_error() {
        None
    } else {
        Some(parser.end())
    }
}

pub struct ParseResult<'a> {
    pub table: Table,
    pub allocator: Allocator,
    pub function: ObjectPtr<FunctionObject<'a>>,
}
