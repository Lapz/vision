use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
};

use crate::{chunk::Chunk, Table, Value};

pub type NativeFn = fn(usize, *const Value) -> Value;
pub type RawObject = *mut Object;
pub type ValuePtr = *mut Value;
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
#[repr(C)]
pub struct UpValueObject {
    _obj: Object,
    pub location: ValuePtr,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct StringObject<'a> {
    _obj: Object,
    pub length: usize,
    pub chars: &'a str,
    pub hash: usize,
}
#[derive(PartialEq)]
#[repr(C)]
pub struct FunctionObject<'a> {
    _obj: Object,
    pub arity: usize,
    pub chunk: Chunk,
    pub upvalue_count: usize,
    pub name: Option<ObjectPtr<StringObject<'a>>>,
}

#[derive(Debug, Clone, PartialEq)]
#[repr(C)]
pub struct NativeObject {
    pub obj: Object,
    pub function: NativeFn,
}
#[derive(Debug)]
#[repr(C)]
pub struct ClosureObject<'a> {
    pub obj: Object,
    pub function: ObjectPtr<FunctionObject<'a>>,
    pub upvalues: Vec<Option<UpValueObject>>,
    pub upvalue_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]

pub enum ObjectType {
    String,
    Function,
    Native,
    Closure,
    UpValue,
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
            upvalue_count: 0,
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

impl<'a> ClosureObject<'a> {
    pub fn new(function: ObjectPtr<FunctionObject<'a>>) -> ObjectPtr<Self> {
        let mut upvalues = Vec::new();

        for i in 0..function.upvalue_count {
            upvalues.push(None)
        }
        ObjectPtr::new(Box::into_raw(Box::new(ClosureObject {
            obj: Object::new(
                ObjectType::Closure,
                std::ptr::null::<RawObject>() as RawObject,
            ),
            upvalue_count: function.upvalue_count,
            upvalues,
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
        #[cfg(debug_assertions)]
        unsafe {
            let obj = *&(*self.as_ptr());
            assert!(obj.ty == ObjectType::Function)
        }
        ObjectPtr::new(self.ptr)
    }

    pub fn as_native(&self) -> ObjectPtr<NativeObject> {
        #[cfg(debug_assertions)]
        unsafe {
            let obj = *&(*self.as_ptr());
            assert!(obj.ty == ObjectType::Native)
        }
        ObjectPtr::new(self.ptr)
    }

    pub fn as_closure<'a>(&self) -> ObjectPtr<ClosureObject<'a>> {
        #[cfg(debug_assertions)]
        unsafe {
            let obj = *&(*self.as_ptr());
            assert!(obj.ty == ObjectType::Closure)
        }
        ObjectPtr::new(self.ptr)
    }

    pub fn take<'a, V: Sized>(self) -> &'a V {
        unsafe { &*(self.ptr as *const V) }
    }

    pub fn is_null(&self) -> bool {
        self.ptr.is_null()
    }
}

impl<'a> UpValueObject {
    pub fn new(location: ValuePtr) -> Self {
        Self {
            _obj: Object::new(
                ObjectType::UpValue,
                std::ptr::null::<RawObject>() as RawObject,
            ),
            location,
        }
    }
}
macro_rules! impl_object_traits {
    ($trait:ident) => {
        impl<'a> AsRef<$trait<'a>> for ObjectPtr<$trait<'a>> {
            fn as_ref(&self) -> &$trait<'a> {
                unsafe { &*(self.ptr as *const $trait<'a>) }
            }
        }

        impl<'a> Into<ObjectPtr<RawObject>> for ObjectPtr<$trait<'a>> {
            fn into(self) -> ObjectPtr<RawObject> {
                ObjectPtr::new(self.ptr)
            }
        }
    };
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

impl<'a> DerefMut for ObjectPtr<ClosureObject<'a>> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *(self.ptr as *mut ClosureObject) }
    }
}

impl_object_traits!(StringObject);
impl_object_traits!(FunctionObject);
impl_object_traits!(ClosureObject);

impl<'a> Into<ObjectPtr<RawObject>> for ObjectPtr<NativeObject> {
    fn into(self) -> ObjectPtr<RawObject> {
        ObjectPtr::new(self.ptr)
    }
}

impl<'a> Clone for ObjectPtr<ClosureObject<'a>> {
    fn clone(&self) -> Self {
        Self {
            ptr: self.ptr,
            tag: self.tag,
        }
    }
}

impl<'a> Clone for ObjectPtr<FunctionObject<'a>> {
    fn clone(&self) -> Self {
        Self {
            ptr: self.ptr,
            tag: self.tag,
        }
    }
}

impl<T: PartialEq + ?Sized + Debug> PartialEq for ObjectPtr<T> {
    fn eq(&self, other: &Self) -> bool {
        self.ptr == other.ptr && self.tag == other.tag
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
