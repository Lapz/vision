use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
};

use crate::{chunk::Chunk, Table, Value};

pub type NativeFn = fn(usize, *const Value) -> Value;
pub type RawObject = *mut Object;
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct Object {
    pub ty: ObjectType,
    pub next: RawObject,
}
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct ObjectPtr<T: ?Sized + Debug> {
    ptr: RawObject,
    tag: std::marker::PhantomData<T>,
}

#[derive(Debug, Clone, Copy, PartialEq)]

pub struct StringObject<'a> {
    _obj: Object,
    pub length: usize,
    pub chars: &'a str,
    pub hash: usize,
}
#[repr(C)]
pub struct FunctionObject<'a> {
    _obj: Object,
    pub arity: usize,
    pub chunk: Chunk,
    pub name: Option<ObjectPtr<StringObject<'a>>>,
}

#[derive(Debug, Clone, PartialEq)]
#[repr(C)]
pub struct NativeObject {
    pub obj: Object,
    pub function: NativeFn,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]

pub enum ObjectType {
    String,
    Function,
    Native,
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

    pub fn new(string: &'a str, table: &mut Table, next: RawObject) -> ObjectPtr<StringObject<'a>> {
        let mut buffer = String::with_capacity(string.len() + 1);

        buffer.push_str(string);
        buffer.push('\0');

        let hash = hash_string(&buffer);
        let length = buffer.len();

        let interned = table.find_string(&buffer, hash);

        if interned.is_some() {
            return ObjectPtr::new(interned.unwrap() as RawObject);
        }

        let s = StringObject {
            _obj: Object::new(ObjectType::String, next),
            hash,
            chars: Box::leak(Box::new(buffer)),
            length,
        };

        let ptr = Box::into_raw(Box::new(s)) as RawObject;

        table.set(ptr, Value::nil());

        ObjectPtr::new(ptr)
    }

    /// Creates a new String Object that takes ownership of the string passed in
    pub fn from_owned(
        chars: String,
        table: &Table,
        next: RawObject,
    ) -> ObjectPtr<StringObject<'a>> {
        let length = chars.len();
        let hash = hash_string(&chars);

        let interned = table.find_string(&chars, hash);

        if interned.is_some() {
            return ObjectPtr::new(interned.unwrap() as RawObject);
        }

        let s = StringObject {
            _obj: Object::new(ObjectType::String, next),
            hash,
            chars: Box::leak(Box::new(chars)),
            length,
        };

        ObjectPtr::new(Box::into_raw(Box::new(s)) as RawObject)
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
    pub fn new(
        name: Option<ObjectPtr<StringObject<'a>>>,
        next: RawObject,
    ) -> ObjectPtr<FunctionObject<'a>> {
        ObjectPtr::new(Box::into_raw(Box::new(Self {
            _obj: Object::new(ObjectType::Function, next),
            arity: 0,
            chunk: Chunk::new(),
            name,
        })) as RawObject)
    }
}

impl NativeObject {
    pub fn new(function: NativeFn) -> ObjectPtr<NativeObject> {
        ObjectPtr::new(Box::into_raw(Box::new(NativeObject {
            obj: Object::new(
                ObjectType::Native,
                std::ptr::null::<RawObject>() as RawObject,
            ),
            function,
        })) as RawObject)
    }
}
impl<T: ?Sized + Debug> ObjectPtr<T> {
    pub fn new(ptr: RawObject) -> ObjectPtr<T> {
        Self {
            ptr,
            tag: std::marker::PhantomData,
        }
    }

    pub fn null() -> ObjectPtr<T> {
        Self {
            ptr: std::ptr::null::<RawObject>() as RawObject,
            tag: std::marker::PhantomData,
        }
    }

    pub fn as_ptr(&self) -> RawObject {
        self.ptr
    }

    pub fn as_ptr_obj(&self) -> ObjectPtr<RawObject> {
        ObjectPtr::new(self.ptr)
    }

    pub fn as_function<'a>(&self) -> ObjectPtr<FunctionObject<'a>> {
        ObjectPtr::new(self.ptr)
    }

    pub fn as_native_function(&self) -> ObjectPtr<NativeObject> {
        ObjectPtr::new(self.ptr)
    }

    pub fn as_native(&self) -> ObjectPtr<NativeObject> {
        ObjectPtr::new(self.ptr)
    }

    pub fn take<'a, V: Sized>(self) -> &'a V {
        unsafe { &*(self.ptr as *const V) }
    }

    pub fn is_null(&self) -> bool {
        self.ptr.is_null()
    }
}

impl<T: Debug> Deref for ObjectPtr<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*(self.ptr as *const T) }
    }
}

impl<'a> DerefMut for ObjectPtr<FunctionObject<'a>> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *(self.ptr as *mut FunctionObject) }
    }
}

impl<'a> AsRef<StringObject<'a>> for ObjectPtr<StringObject<'a>> {
    fn as_ref(&self) -> &StringObject<'a> {
        unsafe { &*(self.ptr as *const StringObject<'a>) }
    }
}

impl<'a> AsRef<FunctionObject<'a>> for ObjectPtr<StringObject<'a>> {
    fn as_ref(&self) -> &FunctionObject<'a> {
        unsafe { &*(self.ptr as *const FunctionObject<'a>) }
    }
}

impl<'a> Into<ObjectPtr<RawObject>> for ObjectPtr<StringObject<'a>> {
    fn into(self) -> ObjectPtr<RawObject> {
        ObjectPtr::new(self.ptr)
    }
}

impl<'a> Into<ObjectPtr<RawObject>> for ObjectPtr<FunctionObject<'a>> {
    fn into(self) -> ObjectPtr<RawObject> {
        ObjectPtr::new(self.ptr)
    }
}

impl<'a> Into<ObjectPtr<RawObject>> for ObjectPtr<NativeObject> {
    fn into(self) -> ObjectPtr<RawObject> {
        ObjectPtr::new(self.ptr)
    }
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
