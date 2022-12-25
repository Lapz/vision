use ast::prelude::Token;
use compiler::{compile, ParseResult};

use std::env;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use vm::{ClosureObject, Value, VM};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let args = env::args().collect::<Vec<String>>();

    let src = "10+10;a+10; -2*3+46; a := 10; a := b := c := d; let a := 10+46;";
    let mut parser = compiler::v2::Parser::new(src);

    while !parser.match_token(Token::Eof) {
        println!("{}", parser.statement());
    }

    // if args.len() == 1 {
    //     repl()?;
    // } else if args.len() == 2 {
    //     run_file(&args[1])?;
    // } else {
    //     println!("Usage: vision [script]");
    //     std::process::exit(64);
    // }

    Ok(())
}

fn interpret(src: &str) -> Result<(), Box<dyn std::error::Error>> {
    let ParseResult {
        function,
        mut allocator,
        table,
    } = compile(src).ok_or("Compile error")?;

    if function.is_null() {
        Err("Compile error".into())
    } else {
        let function_ptr = function.as_function();

        let closure = allocator.alloc(|next| ClosureObject::new(function_ptr, next));

        let mut vm = VM::new(table, allocator);

        vm.push(Value::object(function.as_ptr_obj()));

        vm.pop();

        vm.push(Value::object(closure.clone().into()));

        vm.call(closure, 0);

        vm.run()
    }
}

fn repl() -> Result<(), Box<dyn std::error::Error>> {
    let mut buffer = String::new();

    loop {
        print!("> ");

        std::io::stdin().read_line(&mut buffer)?;

        interpret(&buffer)?;
        break;
    }

    Ok(())
}

fn run_file(path: &dyn AsRef<Path>) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::open(path)?;

    let mut buffer = String::with_capacity(1024);

    file.read_to_string(&mut buffer)?;

    interpret(&buffer)
}

#[cfg(test)]
mod tests {
    use crate::interpret;

    #[test]
    fn it_works() {
        interpret(
            r#"
        var globalSet;
        var globalGet;

        fun main() {
        var a = "initial";

        fun set() { a = "updated"; }
        fun get() { print a; }

        globalSet = set;
        globalGet = get;
        }

        main();
        globalSet();
        globalGet();
    "#,
        )
        .unwrap();
    }
}
