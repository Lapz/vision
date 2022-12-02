use crate::{chunk::Chunk, op, value::Value};
use std::fmt;
pub const STACK_MAX: usize = 256;

pub struct VM {
    chunk: Chunk,
    stack: [Value; STACK_MAX],
    stack_top: usize,
    ip: usize,
}
#[derive(Debug)]
pub enum Error {
    CompileError(String),
    RuntimeError(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::CompileError(e) => write!(f, "Compile error: {}", e),
            Error::RuntimeError(e) => write!(f, "Runtime error: {}", e),
        }
    }
}

impl std::error::Error for Error {}

macro_rules! read_byte {
    ($vm:ident) => {{
        let temp = $vm.ip;
        $vm.ip += 1;

        $vm.chunk[temp]
    }};
}

macro_rules! read_constant {
    ($vm:ident) => {{
        $vm.chunk.constants[read_byte!($vm) as usize]
    }};
}

macro_rules! binary_op {
    ($op:tt,$_self:ident) => {{


            let b = $_self.pop();

            let a = $_self.pop();

            $_self.push(a $op b)

    }};
}

impl VM {
    pub fn new() -> Self {
        Self {
            chunk: Chunk::new(),
            stack: [f64::default(); STACK_MAX],
            stack_top: 0,
            ip: 0,
        }
    }

    pub fn interpret(&mut self, chunk: Chunk) -> Result<(), Box<dyn std::error::Error>> {
        self.chunk = chunk;
        return self.run();
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            let instruction = read_byte!(self);

            #[cfg(feature = "trace_execution")]
            {
                print!("          ");
                for slot in 0..self.stack_top {
                    print!("[ ");
                    print!("{}", self.stack[slot]);
                    print!(" ]");
                }
                println!();
                self.chunk.disassemble_instruction(self.ip - 1);
            }

            match instruction {
                op::RETURN => {
                    return Ok(());
                }
                op::NEGATE => {
                    let value = self.pop();
                    self.push(-value);
                }
                op::CONSTANT => {
                    let constant = read_constant!(self);
                    self.push(constant);
                }
                op::ADD => binary_op!(+ , self),
                op::SUBTRACT => binary_op!(- , self),
                op::MULTIPLY => binary_op!(* , self),
                op::DIVIDE => binary_op!(/ , self),

                _ => return Err(Box::new(Error::RuntimeError("Unknown opcode".to_string()))),
            }
        }
    }

    fn push(&mut self, val: Value) {
        self.stack[self.stack_top] = val;
        self.stack_top += 1;
    }

    fn pop(&mut self) -> Value {
        self.stack_top -= 1;
        self.stack[self.stack_top]
    }
}
