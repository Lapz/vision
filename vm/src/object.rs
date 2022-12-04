#[derive(Clone, Copy)]
#[repr(C)]
pub struct Object {
    pub ty: ObjectType,
    pub next: RawObject,
}

pub type RawObject = *mut Object;

pub struct StringObject<'a> {
    _obj: Object,
    pub length: usize,
    pub chars: &'a str,
}

#[derive(Clone, Copy, PartialEq, Eq)]

pub enum ObjectType {
    String,
}

impl Object {
    pub fn new(ty: ObjectType, next: RawObject) -> Self {
        Object { ty, next }
    }
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
            chars: Box::leak(Box::new(buffer)),
            length,
        };

        Box::into_raw(Box::new(s)) as RawObject
    }

    /// Creates a new String Object that takes ownership of the string passed in
    pub fn from_owned(chars: String, next: RawObject) -> RawObject {
        let length = chars.len();

        let s = StringObject {
            _obj: Object::new(ObjectType::String, next),
            chars: Box::leak(Box::new(chars)),
            length,
        };

        Box::into_raw(Box::new(s)) as RawObject
    }

    pub fn value(&self) -> &str {
        self.chars
    }
}
