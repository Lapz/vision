use compiler::compile;
use std::env;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use vm::{chunk::Chunk, op, VM};

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

    let mut vm = VM::new();

    Ok(())
}

fn interpret(src: &str) -> Result<(), Box<dyn std::error::Error>> {
    compile(src);
    Ok(())

    // let mut vm = VM::new();

    // let mut chunk = Chunk::new();
    // vm.interpret(chunk)
}

fn repl() -> Result<(), Box<dyn std::error::Error>> {
    let mut buffer = String::new();

    loop {
        print!("> ");

        std::io::stdin().read_line(&mut buffer)?;

        interpret(&buffer)?;
    }
}

fn run_file(path: &dyn AsRef<Path>) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::open(path)?;

    let mut buffer = String::with_capacity(1024);

    file.read_to_string(&mut buffer)?;

    interpret(&buffer)
}
