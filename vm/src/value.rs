use std::fmt::Debug;

use crate::{
    object::{ObjectType, StringObject},
    ClosureObject, FunctionObject, NativeObject, ObjectPtr, RawObject, ValuePtr,
};

#[derive(Clone, Copy)]
/// A value within the VM
pub struct Value {
    pub ty: ValueType,
    repr: As,
}

impl Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Value")
            .field("ty", &self.ty)
            .field("repr", {
                &match self.ty {
                    ValueType::Bool => self.as_bool_ref().to_string(),
                    ValueType::Nil => "nil".to_string(),
                    ValueType::Number => self.as_number_ref().to_string(),
                    ValueType::Object => match self.obj_type() {
                        ObjectType::String => {
                            format!("{:?}", self.as_string().chars)
                        }
                        ObjectType::Function => "<fn> ".to_string(),
                        ObjectType::Native => "<native fn>".to_string(),
                        ObjectType::Closure => "<closure fn>".to_string(),
                        ObjectType::UpValue => "upvalue".to_string(),
                    },
                }
            })
            .finish()
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
pub union As {
    boolean: bool,
    number: f64,
    object: ObjectPtr<RawObject>,
}
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum ValueType {
    Bool,
    Nil,
    Number,
    Object,
}

impl Value {
    #[inline]
    pub fn bool(value: bool) -> Value {
        Value {
            repr: As { boolean: value },
            ty: ValueType::Bool,
        }
    }
    #[inline]
    pub fn nil() -> Value {
        Value {
            repr: As { number: 0.0 },
            ty: ValueType::Nil,
        }
    }
    #[inline]
    pub fn number(value: f64) -> Value {
        Value {
            repr: As { number: value },
            ty: ValueType::Number,
        }
    }

    #[inline]
    pub fn object(object: ObjectPtr<RawObject>) -> Value {
        Value {
            repr: As { object },
            ty: ValueType::Object,
        }
    }

    #[inline]
    pub fn as_bool(&self) -> bool {
        debug_assert_eq!(
            self.ty,
            ValueType::Bool,
            "Value is type `{:?}` instead of {:?}",
            self.ty,
            ValueType::Bool
        );
        unsafe { self.repr.boolean }
    }

    #[inline]
    pub fn as_bool_ref(&self) -> &bool {
        debug_assert_eq!(
            self.ty,
            ValueType::Bool,
            "Value is type `{:?}` instead of {:?}",
            self.ty,
            ValueType::Bool
        );
        unsafe { &self.repr.boolean }
    }

    #[inline]
    pub fn as_number(&self) -> f64 {
        debug_assert_eq!(
            self.ty,
            ValueType::Number,
            "Value is type `{:?}` instead of {:?}",
            self.ty,
            ValueType::Number
        );
        unsafe { self.repr.number }
    }

    pub fn as_number_ref(&self) -> &f64 {
        debug_assert_eq!(
            self.ty,
            ValueType::Number,
            "Value is type `{:?}` instead of {:?}",
            self.ty,
            ValueType::Number
        );
        unsafe { &self.repr.number }
    }

    #[inline]
    pub fn as_obj(&self) -> RawObject {
        unsafe { self.repr.object.as_ptr() }
    }

    #[inline]
    pub fn as_string<'a>(&self) -> ObjectPtr<StringObject<'a>> {
        #[cfg(debug_assertions)]
        {
            let repr = self.as_obj();

            let ty = unsafe { (*repr).ty };

            debug_assert_eq!(
                ObjectType::String,
                ty,
                "Value is type `{:?}` instead of {:?}",
                ty,
                ObjectType::String,
            );
        }
        unsafe { self.repr.object.cast() }
    }

    #[inline]
    pub fn as_function<'a>(&self) -> ObjectPtr<FunctionObject<'a>> {
        #[cfg(debug_assertions)]
        {
            let repr = self.as_obj();

            let ty = unsafe { (*repr).ty };

            debug_assert_eq!(
                ObjectType::Function,
                ty,
                "Value is type `{:?}` instead of {:?}",
                ty,
                ObjectType::Function,
            );
        }
        unsafe { self.repr.object.cast() }
    }

    #[inline]
    pub fn as_native(&self) -> ObjectPtr<NativeObject> {
        #[cfg(debug_assertions)]
        {
            let repr = self.as_obj();

            let ty = unsafe { (*repr).ty };

            debug_assert_eq!(
                ObjectType::Native,
                ty,
                "Value is type `{:?}` instead of {:?}",
                ty,
                ObjectType::Native
            );
        }
        unsafe { self.repr.object.cast() }
    }

    #[inline]
    pub fn as_closure<'a>(&self) -> ObjectPtr<ClosureObject<'a>> {
        #[cfg(debug_assertions)]
        {
            let repr = self.as_obj();

            let ty = unsafe { (*repr).ty };

            debug_assert_eq!(
                ObjectType::Closure,
                ty,
                "Value is type `{:?}` instead of {:?}",
                ty,
                ObjectType::Closure,
            );
        }
        unsafe { self.repr.object.cast() }
    }

    #[inline]
    pub fn as_raw_string<'a>(&self) -> &'a str {
        let ptr = self.as_obj();
        unsafe {
            let string_object = &*(ptr as *const StringObject<'a>);
            string_object.chars
        }
    }

    #[inline]
    pub fn is_bool(&self) -> bool {
        self.ty == ValueType::Bool
    }
    #[inline]

    pub fn is_nil(&self) -> bool {
        self.ty == ValueType::Nil
    }
    #[inline]
    pub fn is_number(&self) -> bool {
        self.ty == ValueType::Number
    }

    #[inline]
    pub fn is_obj(&self) -> bool {
        self.ty == ValueType::Object
    }

    #[inline]
    pub fn is_function(&self) -> bool {
        self.is_obj_type(ObjectType::Function)
    }

    #[inline]
    pub fn is_native(&self) -> bool {
        self.is_obj_type(ObjectType::Native)
    }

    #[inline]
    pub fn is_falsey(&self) -> bool {
        self.is_nil() || (self.is_bool() && !self.as_bool())
    }

    #[inline]
    pub fn obj_type(&self) -> ObjectType {
        unsafe { (*self.as_obj()).ty }
    }

    #[inline]
    pub fn is_string(&self) -> bool {
        self.is_obj_type(ObjectType::String)
    }
    #[inline]

    pub fn is_obj_type(&self, ty: ObjectType) -> bool {
        self.is_obj() && self.obj_type() == ty
    }

    #[inline]
    pub fn as_ptr(&self) -> ValuePtr {
        self as ValuePtr
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        if self.ty != other.ty {
            return false;
        }
        match self.ty {
            ValueType::Bool => self.as_bool() == other.as_bool(),
            ValueType::Nil => true,
            ValueType::Number => self.as_number() == other.as_number(),
            ValueType::Object => self.as_obj() == other.as_obj(),
        }
    }
}
