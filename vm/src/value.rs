use crate::{
    object::{Object, ObjectType, StringObject},
    RawObject,
};

#[derive(Clone, Copy)]
/// A value within the VM
pub struct Value {
    pub ty: ValueType,
    repr: As,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub union As {
    boolean: bool,
    number: f64,
    object: RawObject,
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
    pub fn object(object: RawObject) -> Value {
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

    #[inline]
    pub fn as_obj(&self) -> RawObject {
        debug_assert_eq!(
            self.ty,
            ValueType::Object,
            "Value is type `{:?}` instead of {:?}",
            self.ty,
            ValueType::Object
        );
        unsafe { self.repr.object }
    }

    #[inline]
    pub fn as_string<'a>(&self) -> &StringObject<'a> {
        let ptr = self.as_obj();
        unsafe { &*(ptr as *const StringObject<'a>) }
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
            ValueType::Object => {
                let as_string = self.as_string();
                let other_string = other.as_string();

                as_string.length == other_string.length && as_string.chars == other_string.chars
            }
        }
    }
}
