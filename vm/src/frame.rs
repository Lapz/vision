use crate::{FunctionObject, ObjectPtr, RawObject};
#[derive(Debug)]
pub struct CallFrame<'a> {
    pub function: ObjectPtr<FunctionObject<'a>>,
    pub ip: usize,
    /// The slots field points into the VM’s value stack at the first slot that this function can use
    pub slots: usize,
}

impl<'a> CallFrame<'a> {
    pub fn new() -> Self {
        Self {
            function: FunctionObject::new_ptr(None, std::ptr::null::<RawObject>() as RawObject),
            ip: 0,
            slots: 0,
        }
    }
}
