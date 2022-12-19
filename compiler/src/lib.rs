mod compiler;
mod parser;
mod scanner;
mod token;

use scanner::Scanner;
use vm::{FunctionObject, ObjectPtr, RawObject, Table};

use crate::token::TokenType;

pub fn compile(input: &str) -> Option<(ObjectPtr<FunctionObject>, Table, RawObject)> {
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
