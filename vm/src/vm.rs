use crate::{
    chunk::Chunk,
    op,
    value::{Value, ValueType},
};
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
    RuntimeError,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::CompileError(e) => write!(f, "Compile error: {}", e),
            Error::RuntimeError => write!(f, "Runtime error"),
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
    ($val_ty:ident,$op:tt,$self:ident) => {{


            if !$self.peek(0).is_number() || !$self.peek(1).is_number()  {
                runtime_error!($self,"Operands must be numbers.");
                return Err(Box::new(Error::RuntimeError))
            }

            let b = $self.pop().as_number();

            let a = $self.pop().as_number();;

            $self.push(Value::$val_ty(a $op b));

    }};
}

macro_rules! runtime_error {
    () => {
        $crate::eprint!("\n")
    };
    ($self:ident,$($arg:tt)*) => {{
        eprintln!($($arg)*);

        let instruction = $self.ip - $self.chunk.code[$self.ip - 1] as usize;

        let line = $self.chunk.lines[instruction];


        eprintln!("[line {}] in script", line);


        $self.reset_stack();


    }};
}

impl VM {
    pub fn new(chunk: Chunk) -> Self {
        Self {
            chunk,
            stack: [Value::nil(); STACK_MAX],
            stack_top: 0,
            ip: 0,
        }
    }

    pub fn interpret(&mut self) -> Result<(), Box<dyn std::error::Error>> {
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
                    print_value(self.stack[slot]);
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
                    if !self.peek(0).is_number() {
                        runtime_error!(self, "Operand must be a number.");

                        return Err(Box::new(Error::RuntimeError));
                    }
                    let value = self.pop();
                    self.push(Value::number(-value.as_number()));
                }
                op::CONSTANT => {
                    let constant = read_constant!(self);

                    print_value(constant);
                    print!("\n");
                    self.push(constant);
                }
                op::GREATER => binary_op!(bool,>, self),
                op::LESS => binary_op!(bool,< , self),
                op::ADD => binary_op!(number,+ , self),
                op::SUBTRACT => binary_op!(number,- , self),
                op::MULTIPLY => binary_op!(number,* , self),
                op::DIVIDE => binary_op!(number,/ , self),
                op::NIL => self.push(Value::nil()),
                op::TRUE => self.push(Value::bool(true)),
                op::FALSE => self.push(Value::bool(false)),
                op::NOT => {
                    let val = Value::bool(self.pop().is_falsey());
                    self.push(val)
                }
                op::EQUAL => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(Value::bool(a == b));
                }

                _ => {
                    runtime_error!(self, "Unknown opcode");

                    return Err(Box::new(Error::RuntimeError));
                }
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

    fn reset_stack(&mut self) {
        self.stack_top = 0;
    }

    fn peek(&self, distance: i32) -> Value {
        self.stack[self.stack_top - 1 - distance as usize]
    }
}

pub fn print_value(value: Value) {
    match value.ty {
        ValueType::Bool => print!("{}", value.as_bool()),
        ValueType::Nil => print!("nil"),
        ValueType::Number => print!("{}", value.as_number()),
    }
}
