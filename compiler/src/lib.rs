mod parser;
mod scanner;
mod token;

use scanner::Scanner;
use vm::{chunk::Chunk, RawObject};

use crate::token::TokenType;

pub fn compile(input: &str) -> Option<(Chunk, RawObject)> {
    let scanner = Scanner::new(input);
    let mut parser = parser::Parser::new(scanner);

    parser.advance();

    parser.expression();

    parser.consume(TokenType::Eof, "Expect end of expression.");

    if parser.had_error {
        None
    } else {
        Some(parser.end())
    }
}
