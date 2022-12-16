use crate::{FunctionObject, RawObject};

pub struct CallFrame<'a> {
    pub function: FunctionObject<'a>,
    pub ip: usize,
    /// The slots field points into the VM’s value stack at the first slot that this function can use
    pub slots: usize,
}

impl<'a> CallFrame<'a> {
    pub fn new() -> Self {
        Self {
            function: FunctionObject::new(None, std::ptr::null::<RawObject>() as RawObject),
            ip: 0,
            slots: 0,
        }
    }
}
