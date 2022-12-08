use std::mem::ManuallyDrop;

#[derive(Debug, Clone, PartialEq)]
#[repr(C)]
pub struct Object {
    pub ty: ObjectType,
    pub next: RawObject,
}

pub type RawObject = *mut Object;
#[derive(Debug, PartialEq)]
pub struct StringObject<'a> {
    _obj: Object,
    pub length: usize,
    pub chars: &'a str,
    pub hash: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]

pub enum ObjectType {
    String,
}

impl Object {
    pub fn new(ty: ObjectType, next: RawObject) -> Self {
        Object { ty, next }
    }
}

fn hash_string(string: &str) -> u32 {
    let mut hash = 2166136261u32;

    for c in string.chars() {
        hash ^= c as u32;
        hash = hash.wrapping_mul(16777619);
    }

    hash as u32
}

impl<'a> StringObject<'a> {
    /// Create a new string Object that dosen't take ownership of the string passed in

    pub fn new(string: &'a str, next: RawObject) -> RawObject {
        let mut buffer = String::with_capacity(string.len() + 1);

        buffer.push_str(string);
        buffer.push('\0');

        let length = buffer.len();

        let s = StringObject {
            _obj: Object::new(ObjectType::String, next),
            hash: hash_string(&buffer),
            chars: Box::leak(Box::new(buffer)),
            length,
        };

        Box::into_raw(Box::new(s)) as RawObject
    }
    pub fn new2(string: &'a str, next: RawObject) -> Self {
        let mut buffer = String::with_capacity(string.len() + 1);

        buffer.push_str(string);
        buffer.push('\0');

        let length = buffer.len();

        let s = StringObject {
            _obj: Object::new(ObjectType::String, next),
            hash: hash_string(&buffer),
            chars: Box::leak(Box::new(buffer)),
            length,
        };

        s
    }

    /// Creates a new String Object that takes ownership of the string passed in
    pub fn from_owned(chars: String, next: RawObject) -> RawObject {
        let length = chars.len();

        let s = StringObject {
            _obj: Object::new(ObjectType::String, next),
            hash: hash_string(&chars),
            chars: Box::leak(Box::new(chars)),
            length,
        };

        Box::into_raw(Box::new(s)) as RawObject
    }

    pub fn value(&self) -> &str {
        self.chars
    }
}
