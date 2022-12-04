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
}
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum ValueType {
    Bool,
    Nil,
    Number,
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
    pub fn is_falsey(&self) -> bool {
        self.is_nil() || (self.is_bool() && !self.as_bool())
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
        }
    }
}
