use crate::{
    chunk::Chunk,
    op,
    value::{Value, ValueType},
    ObjectType, RawObject, StringObject, Table,
};
use std::fmt;
pub const STACK_MAX: usize = 256;

pub struct VM {
    chunk: Chunk,
    stack: [Value; STACK_MAX],
    stack_top: usize,
    ip: usize,
    objects: RawObject,
    strings: Table,
    globals: Table,
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

            let a = $self.pop().as_number();

            $self.push(Value::$val_ty(a $op b));

    }};
}

macro_rules! runtime_error {
    () => {
        $crate::eprint!("\n")
    };
    ($self:ident,$($arg:tt)*) => {{
        eprint!($($arg)*);

        let instruction = $self.ip - $self.chunk.code[$self.ip - 1] as usize;

        let line = $self.chunk.lines[instruction];


        eprintln!(" [line {}] in script", line);


        $self.reset_stack();


    }};
}

impl VM {
    pub fn new(chunk: Chunk, strings: Table, objects: RawObject) -> Self {
        Self {
            chunk,
            stack: [Value::nil(); STACK_MAX],
            stack_top: 0,
            ip: 0,
            objects,
            strings,
            globals: Table::new(),
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
                    #[cfg(feature = "debug")]
                    {
                        print_value(constant);
                        print!("\n");
                    }
                    self.push(constant);
                }
                op::GREATER => binary_op!(bool,>, self),
                op::LESS => binary_op!(bool,< , self),
                op::ADD => {
                    if self.peek(0).is_string() && self.peek(1).is_string() {
                        self.concatenate();
                    } else if self.peek(0).is_number() && self.peek(1).is_number() {
                        let b = self.pop();
                        let a = self.pop();

                        self.push(Value::number(a.as_number() + b.as_number()));
                    } else {
                        runtime_error!(self, "Operands must be two numbers or two strings.");
                        return Err(Box::new(Error::RuntimeError));
                    }
                }
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
                op::PRINT => {
                    let val = self.pop();
                    print_value(val);
                    print!("\n");
                }
                op::POP => {
                    self.pop();
                }

                op::DEFINE_GLOBAL => {
                    let name = read_constant!(self).as_obj();
                    let val = self.peek(0);
                    self.globals.set(name, val);

                    self.pop();
                }
                op::GET_GLOBAL => {
                    let val = read_constant!(self);

                    let obj_ptr = val.as_obj();

                    let as_str = val.as_string();

                    let val = self.globals.get(obj_ptr);

                    if val.is_none() {
                        runtime_error!(self, "Undefined variable '{}'", as_str.chars);
                        return Err(Box::new(Error::RuntimeError));
                    }

                    self.push(val.unwrap());
                }

                op::SET_GLOBAL => {
                    let global_val = read_constant!(self);

                    let obj_ptr = global_val.as_obj();

                    let as_str = global_val.as_string();

                    let value = self.peek(0);

                    if self.globals.set(obj_ptr, value) {
                        runtime_error!(self, "Undefined variable '{}'", as_str.chars);
                        return Err(Box::new(Error::RuntimeError));
                    }

                    // self.push(val.unwrap());
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

    fn concatenate(&mut self) {
        let b = self.pop();
        let a = self.pop();

        let mut new_string = String::with_capacity(
            b.as_string().chars.len() - 1 + a.as_string().chars.len() - 1 + 1,
        );
        // We don't include the null terminator in the length of the string.
        new_string.push_str(&a.as_string().chars[0..a.as_string().chars.len() - 1]);

        new_string.push_str(&b.as_string().chars[0..b.as_string().chars.len() - 1]);
        new_string.push('\0');

        let result = Value::object(StringObject::from_owned(
            new_string,
            &mut self.strings,
            self.objects,
        ));
        self.push(result);
    }
}

pub fn print_value(value: Value) {
    match value.ty {
        ValueType::Bool => print!("{}", value.as_bool()),
        ValueType::Nil => print!("nil"),
        ValueType::Number => print!("{}", value.as_number()),
        ValueType::Object => print_object(value),
    }
}

#[inline]
pub fn print_object(value: Value) {
    match value.obj_type() {
        ObjectType::String => print!("{}", value.as_raw_string()),
    }
}

fn free_object(obj: RawObject) {
    let obj_obj = unsafe { &*(obj) };
    match obj_obj.ty {
        ObjectType::String => unsafe {
            let _ = Box::from_raw(obj);
        },
    }
}

impl Drop for VM {
    fn drop(&mut self) {
        let mut obj = self.objects;

        while !obj.is_null() {
            #[cfg(feature = "debug")]
            {
                print!("Freeing object ");
                print_object(Value::object(obj));
                print!("\n");
            }

            unsafe {
                let next = (&*obj).next;

                let _ = free_object(obj);

                obj = next;
            }
        }
    }
}
