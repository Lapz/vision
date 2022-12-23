use crate::{ObjectPtr, RawObject};
use std::fmt::Debug;
#[derive(Debug, Clone, Copy)]
pub struct Allocator {
    root: RawObject,
}

impl Allocator {
    pub fn new() -> Self {
        let root = std::ptr::null::<RawObject>() as RawObject;

        Self { root: root }
    }

    pub fn alloc<T: ?Sized + Debug, F: FnOnce(RawObject) -> ObjectPtr<T>>(
        &mut self,
        init_obj: F,
    ) -> ObjectPtr<T> {
        println!("{:}", line!());
        let allocated_obj = init_obj(self.root);

        self.root = allocated_obj.raw();

        allocated_obj
    }

    pub fn finish(self) -> RawObject {
        self.root
    }
}

#[cfg(test)]
mod test {
    use super::Allocator;
    use crate::FunctionObject;

    #[test]
    fn it_works() {
        let mut alloc = Allocator::new();

        alloc.alloc(|next| FunctionObject::new(None, next));
        alloc.alloc(|next| FunctionObject::new(None, next));

        let mut root = alloc.finish();

        let mut count = 0;

        while !root.is_null() {
            unsafe {
                let next = (&*root).next;

                // let _ = free_object(obj);

                root = next;
                count += 1;
            }
        }

        assert_eq!(count, 2)
    }
}
