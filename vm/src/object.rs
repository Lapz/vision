use std::fmt::Debug;

use crate::{chunk::Chunk, Table, Value};

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct Object {
    pub ty: ObjectType,
    pub next: RawObject,
}

pub type RawObject = *mut Object;
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StringObject<'a> {
    _obj: Object,
    pub length: usize,
    pub chars: &'a str,
    pub hash: usize,
}

pub struct FunctionObject<'a> {
    _obj: Object,
    pub arity: usize,
    pub chunk: Chunk,
    pub name: Option<StringObject<'a>>,
}

impl<'a> Debug for FunctionObject<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FunctionObject")
            .field("_obj", &self._obj)
            .field("arity", &self.arity)
            .field("name", &self.name)
            .finish()
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]

pub enum ObjectType {
    String,
    Function,
}

impl Object {
    pub fn new(ty: ObjectType, next: RawObject) -> Self {
        Object { ty, next }
    }
}

fn hash_string(string: &str) -> usize {
    let mut hash = 2166136261usize;

    for c in string.chars() {
        hash ^= c as usize;
        hash = hash.wrapping_mul(16777619);
    }

    hash
}

impl<'a> StringObject<'a> {
    /// Create a new string Object that dosen't take ownership of the string passed in

    pub fn new(string: &'a str, table: &mut Table, next: RawObject) -> RawObject {
        let mut buffer = String::with_capacity(string.len() + 1);

        buffer.push_str(string);
        buffer.push('\0');

        let hash = hash_string(&buffer);
        let length = buffer.len();

        let interned = table.find_string(&buffer, hash);

        if interned.is_some() {
            return interned.unwrap() as RawObject;
        }

        let s = StringObject {
            _obj: Object::new(ObjectType::String, next),
            hash,
            chars: Box::leak(Box::new(buffer)),
            length,
        };

        let ptr = Box::into_raw(Box::new(s)) as RawObject;

        table.set(ptr, Value::nil());

        ptr
    }

    /// Creates a new String Object that takes ownership of the string passed in
    pub fn from_owned(chars: String, table: &Table, next: RawObject) -> RawObject {
        let length = chars.len();
        let hash = hash_string(&chars);

        let interned = table.find_string(&chars, hash);

        if interned.is_some() {
            return interned.unwrap();
        }

        let s = StringObject {
            _obj: Object::new(ObjectType::String, next),
            hash,
            chars: Box::leak(Box::new(chars)),
            length,
        };

        Box::into_raw(Box::new(s)) as RawObject
    }

    pub fn to_raw(&self) -> RawObject {
        let ptr: *const StringObject = self;

        ptr as RawObject
    }

    pub fn value(&self) -> &str {
        self.chars
    }
}

impl<'a> FunctionObject<'a> {
    pub fn new(name: Option<StringObject<'a>>, next: RawObject) -> Self {
        Self {
            _obj: Object::new(ObjectType::Function, next),
            arity: 0,
            chunk: Chunk::new(),
            name,
        }
    }

    pub fn to_raw(&self) -> RawObject {
        let ptr: *const FunctionObject = self;

        ptr as RawObject
    }
}
