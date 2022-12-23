use crate::{Allocator, ClosureObject, FunctionObject, ObjectPtr, RawObject};
#[derive(Debug)]
pub struct CallFrame<'a> {
    pub closure: ObjectPtr<ClosureObject<'a>>,
    pub ip: usize,
    /// The slots field points into the VM’s value stack at the first slot that this function can use
    pub slots: usize,
}

impl<'a> CallFrame<'a> {
    pub fn new(allocator: &mut Allocator) -> Self {
        let fn_object = allocator.alloc(|next| FunctionObject::new(None, next));

        let closure = allocator.alloc(move |next| ClosureObject::new(fn_object, next));
        Self {
            closure,
            ip: 0,
            slots: 0,
        }
    }
}
