use crate::{
    frame::CallFrame,
    op,
    value::{Value, ValueType},
    FunctionObject, ObjectPtr, ObjectType, RawObject, StringObject, Table,
};
use std::fmt;
pub const STACK_MAX: usize = FRAMES_MAX * (u8::BITS as usize);
pub const FRAMES_MAX: usize = 64;

pub struct VM<'a> {
    stack: [Value; STACK_MAX],
    pub frames: Vec<CallFrame<'a>>,
    pub stack_top: usize,
    pub frame_count: usize,
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

macro_rules! frame {
    ($vm:ident) => {{
        let frame = $vm.frames.get($vm.frame_count - 1).expect("No frame found");
        frame
    }};

    ($vm:ident,$index:expr) => {{
        let frame = $vm.frames.get($index).expect("No frame found");
        frame
    }};
}

macro_rules! frame_mut {
    ($vm:ident) => {{
        let frame = $vm
            .frames
            .get_mut($vm.frame_count - 1)
            .expect("No frame found");
        frame
    }};
}

macro_rules! read_byte {
    ($vm:ident) => {{
        let frame = frame_mut!($vm);

        let temp = frame.ip;
        frame.ip += 1;

        frame.function.chunk[temp]
    }};
}

macro_rules! read_short {
    ($vm:ident) => {{
        let frame = frame_mut!($vm);

        frame.ip += 2;

        let temp = frame.ip;

        (frame.function.chunk[temp - 2] as u16) << 8 | frame.function.chunk[temp - 1] as u16
    }};
}

macro_rules! read_constant {
    ($vm:ident) => {{
        let byte = read_byte!($vm) as usize;
        let frame = frame!($vm);

        frame.function.chunk.constants[byte]
    }};
}

macro_rules! binary_op {
    ($val_ty:ident,$op:tt,$self:ident) => {{
        if !$self.peek(0).is_number() || !$self.peek(1).is_number() {
            runtime_error!($self, "Operands must be numbers.");
            return Err(Box::new(Error::RuntimeError));
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
        eprintln!("");
        eprintln!($($arg)*);


        for i in (0..$self.frame_count).rev() {
            let frame = frame!($self,i);
            let instruction = frame.ip;
            let line = frame.function.chunk.lines[instruction];
            eprint!(" [line {}] in ", line);
            if frame.function.name.is_none() {
                eprintln!("script");
            }else{
                eprintln!("{}()",frame.function.name.unwrap().chars)
            }

        }

        $self.reset_stack();


    }};
}

impl<'a> VM<'a> {
    pub fn new(strings: Table, objects: RawObject) -> Self {
        let mut frames = Vec::new();

        for _ in 0..FRAMES_MAX {
            frames.push(CallFrame::new())
        }

        Self {
            stack: [Value::nil(); STACK_MAX],
            frames,
            stack_top: 0,
            frame_count: 0,

            objects,
            strings,
            globals: Table::new(),
        }
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            let instruction = read_byte!(self);

            {
                #[cfg(feature = "trace")]
                {
                    print!("          ");
                    for slot in 0..self.stack_top {
                        print!("[ ");
                        print_value(self.stack[slot]);
                        print!(" ]");
                    }
                    println!();
                }

                #[cfg(feature = "debug")]
                {
                    let frame = frame!(self);

                    frame.function.chunk.disassemble_instruction(frame.ip - 1);
                }
            }

            match instruction {
                op::RETURN => {
                    let result = self.pop();

                    self.frame_count -= 1;

                    if self.frame_count == 1 {
                        self.pop();
                        return Ok(());
                    }

                    let frame = frame!(self);

                    self.stack_top = frame.slots;

                    self.push(result);
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

                op::GET_LOCAL => {
                    let slot = read_byte!(self);
                    let index = frame!(self).slots + slot as usize;
                    self.push(self.stack[index])
                }

                op::SET_LOCAL => {
                    let slot = read_byte!(self);

                    let val = self.peek(0);

                    let index = frame!(self).slots + slot as usize;

                    self.stack[index] = val;
                }
                op::JUMP_IF_FALSE => {
                    let offset = read_short!(self) as usize;

                    let if_false = self.peek(0).is_falsey();
                    if if_false {
                        frame_mut!(self).ip += offset;
                    }
                }

                op::JUMP => {
                    let offset = read_short!(self) as usize;

                    frame_mut!(self).ip += offset;
                }

                op::LOOP => {
                    let offset = read_short!(self) as usize;

                    frame_mut!(self).ip -= offset;
                }

                op::CALL => {
                    let arg_count = read_byte!(self);

                    let callee = self.peek(0);

                    if !self.call_value(callee, arg_count) {
                        return Err(Box::new(Error::RuntimeError));
                    }
                }

                _ => {
                    runtime_error!(self, "Unknown opcode");

                    return Err(Box::new(Error::RuntimeError));
                }
            }
        }
    }

    pub fn push(&mut self, val: Value) {
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
    fn peek(&self, distance: usize) -> Value {
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

        let result = Value::object(
            StringObject::from_owned(new_string, &mut self.strings, self.objects).into(),
        );
        self.push(result);
    }

    fn call_value(&mut self, callee: Value, arg_count: u8) -> bool {
        if callee.is_obj() {
            match callee.obj_type() {
                ObjectType::String => {}
                ObjectType::Function => return self.call(callee.as_function(), arg_count),
            }
        }
        runtime_error!(self, "Can only call functions and classes.");
        false
    }

    pub fn call(&mut self, callee: ObjectPtr<FunctionObject<'a>>, arg_count: u8) -> bool {
        if self.frame_count == FRAMES_MAX {
            runtime_error!(self, "Stack overflow.");
            return false;
        }

        self.frame_count += 1;

        let frame = frame_mut!(self);

        if arg_count as usize != callee.arity {
            runtime_error!(
                self,
                "Expected {} arguments but got {}",
                callee.arity,
                arg_count
            );

            return false;
        }

        frame.function = callee;
        frame.slots = self.stack_top - arg_count as usize - 1;

        true
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
        ObjectType::Function => print_function(&value.as_function()),
    }
}

fn print_function(function: &FunctionObject) {
    match &function.name {
        Some(name) => {
            print!("<fn {}>", name.chars)
        }
        None => {
            print!("<script>")
        }
    }
}

unsafe fn free_object(obj: RawObject) {
    let obj_obj = &*(obj);
    match obj_obj.ty {
        ObjectType::String => {
            let _ = Box::from_raw(obj);
        }
        ObjectType::Function => {
            let _ = Box::from_raw(obj);
        }
    }
}

impl<'a> Drop for VM<'a> {
    fn drop(&mut self) {
        let mut obj = self.objects;

        while !obj.is_null() {
            #[cfg(feature = "debug")]
            {
                print!("Freeing object ");
                // print_object(Value::object(obj));
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
