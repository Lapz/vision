use compiler::compile;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::{env, ptr};
use vm::RawObject;
use vm::{chunk::Chunk, op, StringObject, Table, Value, VM};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = env::args().collect::<Vec<String>>();

    println!("{:?}", args);

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
    let (chunk, object_list) = compile(src).ok_or("Compile error")?;

    let mut vm = VM::new(chunk, object_list);

    let mut table = Table::new();

    table.set(
        StringObject::new2("foo", ptr::null::<RawObject>() as RawObject),
        Value::number(1.4434),
    );
    table.set(
        StringObject::new2("foo1", ptr::null::<RawObject>() as RawObject),
        Value::number(1.2),
    );
    table.set(
        StringObject::new2("foo2", ptr::null::<RawObject>() as RawObject),
        Value::number(1.2),
    );
    table.set(
        StringObject::new2("foo3", ptr::null::<RawObject>() as RawObject),
        Value::bool(true),
    );
    table.set(
        StringObject::new2("foo4", ptr::null::<RawObject>() as RawObject),
        Value::number(1.2),
    );
    table.set(
        StringObject::new2("foo5", ptr::null::<RawObject>() as RawObject),
        Value::number(1.2),
    );
    table.set(
        StringObject::new2("foo6", ptr::null::<RawObject>() as RawObject),
        Value::number(1.2),
    );
    table.set(
        StringObject::new2("foo7", ptr::null::<RawObject>() as RawObject),
        Value::number(1.2),
    );

    table.set(
        StringObject::new2("foo8", ptr::null::<RawObject>() as RawObject),
        Value::number(1.2),
    );

    println!("{:#?}", table);

    vm.interpret()
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
