use compiler::compile;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::{env, error::Error};
use vm::{FunctionObject, ObjectPtr, Value, VM};

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
    let (function, table, object_list) = compile(src).ok_or("Compile error")?;

    if function.is_null() {
        Err("Compile error".into())
    } else {
        let mut vm = VM::new(table, object_list);

        vm.push(Value::object(function.as_ptr_obj()));

        vm.call(function.as_function(), 0);

        let index = vm.frame_count;

        vm.frame_count += 1;

        let frame = vm.frames.get_mut(index).unwrap();

        frame.function = function;

        frame.ip = 0;

        frame.slots = vm.stack_top;

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
