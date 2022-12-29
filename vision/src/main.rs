use compiler::{compile, ParseResult};
use syntax::Parser;

use core::construct_ir;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::{env, process::exit};
use vm::{ClosureObject, Value, VM};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = env::args().collect::<Vec<String>>();

    if args.len() == 1 {
        repl()?;
    } else if args.len() == 2 {
        run_file(&args[1])?;
    } else {
        println!("Usage: vision [script]");
        std::process::exit(64);
    }

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

    let parser = Parser::new(&buffer);

    let ast = match parser.parse() {
        Some(program) => program,
        None => exit(1),
    };

    let ir = construct_ir(&buffer, ast);

    Ok(())
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
