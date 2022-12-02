mod scanner;
mod token;

use scanner::Scanner;

use crate::token::TokenType;

pub fn compile(input: &str) {
    let mut scanner = Scanner::new(input);

    let mut line: usize = 0;

    loop {
        let token = scanner.scan_token();

        if token.line != line {
            print!("{:4} ", token.line);
            line = token.line;
        } else {
            print!("   | ");
        }

        println!("{:2?} {:<12} {}", token.ty, token.length, token.lexme);

        if token.ty == TokenType::Eof {
            break;
        }
    }
}
