use crate::{
    frame::CallFrame,
    native::clock_native,
    op::Op,
    value::{Value, ValueType},
    ClosureObject, FunctionObject, NativeFn, NativeObject, ObjectPtr, ObjectType, RawObject,
    StringObject, Table, UpValueObject, ValuePtr,
};
use std::fmt::Display;
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
    pub open_upvalues: ObjectPtr<UpValueObject>,
}

#[derive(Debug)]
pub enum Error {
    CompileError(String),
    RuntimeError,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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

        frame.closure.function.chunk[temp]
    }};
}

macro_rules! read_short {
    ($vm:ident) => {{
        let frame = frame_mut!($vm);

        frame.ip += 2;

        let temp = frame.ip;

        (frame.closure.function.chunk[temp - 2] as u16) << 8
            | frame.closure.function.chunk[temp - 1] as u16
    }};
}

macro_rules! read_constant {
    ($vm:ident) => {{
        let byte = read_byte!($vm) as usize;
        let frame = frame!($vm);

        frame.closure.function.chunk.constants[byte]
    }};
}

macro_rules! binary_op {
    ($val_ty:ident,$op:tt,$self:ident) => {{

        if !$self.peek(0).is_number() || !$self.peek(1).is_number() {
            runtime_error!($self, "{} operands must be numbers",stringify!($op));
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
            let line = frame.closure.function.chunk.lines[instruction];
            eprint!(" [line {}] in ", line);
            if frame.closure.function.name.is_none() {
                eprintln!("script");
            }else{
                eprintln!("{}()",frame.closure.function.name.unwrap().chars)
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

        let mut vm = Self {
            stack: [Value::nil(); STACK_MAX],
            frames,
            stack_top: 0,
            frame_count: 0,
            objects,
            strings,
            globals: Table::new(),
            open_upvalues: ObjectPtr::null(),
        };

        vm.define_native("clock", clock_native);

        vm
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

                    frame
                        .closure
                        .function
                        .chunk
                        .disassemble_instruction(frame.ip - 1);
                }
            }

            unsafe {
                match std::mem::transmute(instruction) {
                    Op::RETURN => {
                        let result = self.pop();

                        let frame = frame!(self);

                        let slot = frame.slots;

                        self.close_upvalue(self.stack[slot].as_ptr());

                        self.frame_count -= 1;

                        if self.frame_count == 0 {
                            self.pop();
                            return Ok(());
                        }

                        self.stack_top = slot;

                        self.push(result);
                    }
                    Op::NEGATE => {
                        if !self.peek(0).is_number() {
                            runtime_error!(self, "Operand must be a number.");

                            return Err(Box::new(Error::RuntimeError));
                        }
                        let value = self.pop();
                        self.push(Value::number(-value.as_number()));
                    }
                    Op::CONSTANT => {
                        let constant = read_constant!(self);
                        #[cfg(feature = "debug")]
                        {
                            print_value(constant);
                            print!("\n");
                        }
                        self.push(constant);
                    }
                    Op::GREATER => binary_op!(bool,>, self),
                    Op::LESS => binary_op!(bool,< , self),
                    Op::ADD => {
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
                    Op::SUBTRACT => binary_op!(number,- , self),
                    Op::MULTIPLY => binary_op!(number,* , self),
                    Op::DIVIDE => binary_op!(number,/ , self),
                    Op::NIL => self.push(Value::nil()),
                    Op::TRUE => self.push(Value::bool(true)),
                    Op::FALSE => self.push(Value::bool(false)),
                    Op::NOT => {
                        let val = Value::bool(self.pop().is_falsey());
                        self.push(val)
                    }
                    Op::EQUAL => {
                        let b = self.pop();
                        let a = self.pop();
                        self.push(Value::bool(a == b));
                    }
                    Op::PRINT => {
                        let val = self.pop();
                        print_value(val);
                        print!("\n");
                    }
                    Op::POP => {
                        self.pop();
                    }

                    Op::DEFINE_GLOBAL => {
                        let name = read_constant!(self).as_obj();
                        let val = self.peek(0);
                        self.globals.set(name, val);

                        self.pop();
                    }
                    Op::GET_GLOBAL => {
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

                    Op::SET_GLOBAL => {
                        let global_val = read_constant!(self);

                        let obj_ptr = global_val.as_obj();

                        let as_str = global_val.as_string();

                        let value = self.peek(0);

                        if self.globals.set(obj_ptr, value) {
                            self.globals.delete(obj_ptr);
                            runtime_error!(self, "Undefined variable '{}'", as_str.chars);
                            return Err(Box::new(Error::RuntimeError));
                        }

                        // self.push(val.unwrap());
                    }

                    Op::GET_LOCAL => {
                        let slot = read_byte!(self);
                        let index = frame!(self).slots + slot as usize;
                        self.push(self.stack[index])
                    }

                    Op::SET_LOCAL => {
                        let slot = read_byte!(self);

                        let val = self.peek(0);

                        let index = frame!(self).slots + slot as usize;

                        self.stack[index] = val;
                    }
                    Op::JUMP_IF_FALSE => {
                        let offset = read_short!(self) as usize;

                        let if_false = self.peek(0).is_falsey();
                        if if_false {
                            frame_mut!(self).ip += offset;
                        }
                    }

                    Op::JUMP => {
                        let offset = read_short!(self) as usize;

                        frame_mut!(self).ip += offset;
                    }

                    Op::LOOP => {
                        let offset = read_short!(self) as usize;

                        frame_mut!(self).ip -= offset;
                    }

                    Op::CALL => {
                        let arg_count = read_byte!(self);

                        let callee = self.peek(arg_count as usize);

                        if !self.call_value(callee, arg_count as usize) {
                            return Err(Box::new(Error::RuntimeError));
                        }
                    }

                    Op::CLOSURE => {
                        let function = read_constant!(self).as_function();
                        let mut closure = ClosureObject::new(function);

                        for i in 0..closure.upvalue_count {
                            let is_local = read_byte!(self);
                            let index = read_byte!(self);

                            if is_local == 1 {
                                let captured_value_index = frame!(self).slots + index as usize;

                                closure.upvalues[i] =
                                    Some(self.capture_value(self.stack[captured_value_index]));
                            } else {
                                let frame = frame!(self);
                                closure.upvalues[i] = frame.closure.upvalues[index as usize]
                            }
                        }

                        self.push(Value::object(closure.into()));
                    }

                    Op::GET_UPVALUE => {
                        let slot = read_byte!(self);

                        let value = frame!(self).closure.upvalues[slot as usize]
                            .unwrap()
                            .location;

                        self.push(value);
                    }

                    Op::SET_UPVALUE => {
                        let slot = read_byte!(self);

                        let value = self.peek(0);

                        frame_mut!(self).closure.upvalues[slot as usize]
                            .unwrap()
                            .location = value;
                    }

                    Op::CLOSE_UPVALUE => {
                        self.close_upvalue(self.stack[self.stack_top - 1].as_ptr());
                        self.pop();
                    }

                    _ => {
                        runtime_error!(self, "Unknown opcode");

                        return Err(Box::new(Error::RuntimeError));
                    }
                }
            }
        }
    }

    pub fn push(&mut self, val: Value) {
        self.stack[self.stack_top] = val;
        self.stack_top += 1;
    }

    pub fn pop(&mut self) -> Value {
        self.stack_top -= 1;
        self.stack[self.stack_top]
    }

    fn reset_stack(&mut self) {
        self.stack_top = 0;
    }

    const fn peek(&self, distance: usize) -> Value {
        self.stack[self.stack_top - 1 - distance as usize]
    }

    fn define_native(&mut self, name: &str, fn_ptr: NativeFn) {
        let name = Value::object(StringObject::new(name, &mut self.strings, self.objects).into());
        self.push(name);

        self.push(Value::object(NativeObject::new(fn_ptr).into()));

        self.globals.set(name.as_obj(), self.stack[1]);

        self.pop();
        self.pop();
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

    fn call_value(&mut self, callee: Value, arg_count: usize) -> bool {
        if callee.is_obj() {
            match callee.obj_type() {
                //we wrap all functions in ClosureObjects so the runtime will never try to invoke a bare FunctionObject anymore
                ObjectType::String | ObjectType::UpValue | ObjectType::Function => {}

                ObjectType::Closure => return self.call(callee.as_closure(), arg_count),
                ObjectType::Native => {
                    let native = callee.as_native();

                    let result = (native.function)(
                        arg_count as usize,
                        self.stack[self.stack_top - arg_count..self.stack_top].as_ptr(),
                    );

                    self.stack_top -= arg_count + 1;

                    self.push(result);

                    return true;
                }
            }
        }

        runtime_error!(self, "Can only call functions and classes.");
        false
    }

    pub fn call(&mut self, callee: ObjectPtr<ClosureObject<'a>>, arg_count: usize) -> bool {
        if self.frame_count == FRAMES_MAX {
            runtime_error!(self, "Stack overflow.");
            return false;
        }

        self.frame_count += 1;

        let frame = frame_mut!(self);

        if arg_count != callee.function.arity {
            runtime_error!(
                self,
                "Expected {} arguments but got {}",
                callee.function.arity,
                arg_count
            );

            return false;
        }

        frame.ip = 0;
        frame.closure = callee;
        frame.slots = self.stack_top - arg_count - 1 as usize;

        true
    }

    fn capture_value(&mut self, local: Value) -> ObjectPtr<UpValueObject> {
        let mut prev_upvalue = ObjectPtr::null();
        let mut upvalue = self.open_upvalues;

        while !upvalue.is_null() && upvalue.location.as_ptr() > local.as_ptr() {
            prev_upvalue = upvalue;
            upvalue = upvalue.next;
        }

        if !upvalue.is_null() && upvalue.location == local {
            return upvalue;
        }

        let mut created_up_value = UpValueObject::new(local);

        created_up_value.next = upvalue;

        if prev_upvalue.is_null() {
            self.open_upvalues = created_up_value;
        } else {
            prev_upvalue.next = created_up_value
        }

        created_up_value
    }

    fn close_upvalue(&mut self, last: ValuePtr) {
        while !self.open_upvalues.is_null() && self.open_upvalues.location.as_ptr() >= last {
            let mut upvalue = self.open_upvalues;

            upvalue.closed = upvalue.location;
            upvalue.location = upvalue.closed;

            self.open_upvalues = upvalue.next;
        }
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
        ObjectType::Native => print!("<native fn>"),
        ObjectType::Closure => print_function(&value.as_closure().function),
        ObjectType::UpValue => print!("upvalue"),
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
        _ => {
            let _ = Box::from_raw(obj);
        }
    }
}

// impl<'a> Drop for VM<'a> {
//     fn drop(&mut self) {
//         let mut obj = self.objects;

//         while !obj.is_null() {
//             #[cfg(feature = "debug")]
//             {
//                 print!("Freeing object ");
//                 // print_object(Value::object(obj));
//                 print!("\n");
//             }

//             unsafe {
//                 let next = (&*obj).next;

//                 let _ = free_object(obj);

//                 obj = next;
//             }
//         }
//     }
// }
