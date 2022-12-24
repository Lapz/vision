use std::{
    borrow::Borrow,
    cell::RefCell,
    collections::HashMap,
    hash::Hash,
    marker::PhantomData,
    ops::{Deref, Index},
    ptr::NonNull,
    rc::Rc,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SymbolId(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LiteralId(u32);

pub trait InternId: Copy + Clone {
    fn id(index: u32) -> Self;
    fn index(&self) -> u32;
}

impl InternId for SymbolId {
    fn id(index: u32) -> Self {
        SymbolId(index)
    }

    fn index(&self) -> u32 {
        self.0
    }
}

impl InternId for LiteralId {
    fn id(index: u32) -> Self {
        LiteralId(index)
    }

    fn index(&self) -> u32 {
        self.0
    }
}

pub struct OwnedPtr<T: ?Sized> {
    ptr: NonNull<T>,
    marker: PhantomData<Box<T>>,
}

impl<T: ?Sized> Deref for OwnedPtr<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { self.as_ref() }
    }
}

impl<T: ?Sized> Borrow<T> for OwnedPtr<T> {
    fn borrow(&self) -> &T {
        self
    }
}

impl<T: ?Sized> OwnedPtr<T> {
    pub fn new(item: Box<T>) -> Self {
        Self {
            ptr: NonNull::new(Box::into_raw(item)).unwrap(),
            marker: PhantomData,
        }
    }

    pub unsafe fn as_ref<'a>(&self) -> &T {
        unsafe { self.ptr.as_ref() }
    }
}

impl<T: ?Sized + Hash> Hash for OwnedPtr<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.ptr.hash(state);
    }
}

impl<T: ?Sized + Eq> Eq for OwnedPtr<T> {}
impl<T: ?Sized + PartialEq> PartialEq for OwnedPtr<T> {
    fn eq(&self, other: &Self) -> bool {
        (**self).eq(&**other)
    }
}
pub struct Interner<K: ?Sized + Hash + PartialEq + Eq, T: InternId> {
    map: HashMap<OwnedPtr<K>, T>,
}

impl<K: ?Sized + Hash + PartialEq + Eq, T: InternId> Interner<K, T> {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn intern(&mut self, item: K) -> T
    where
        K: Borrow<K> + Into<Box<K>>,
    {
        let borrowed = item.borrow();

        if let Some((_, id)) = self.map.get_key_value(borrowed) {
            return *id;
        }

        let id = T::id(self.map.borrow().len() as u32);

        let key = OwnedPtr::new(item.into());

        self.map.insert(key, id);

        id
    }

    pub fn get(&self, key: &T) -> Option<&K> {
        self.map
            .iter()
            .skip(key.index() as usize)
            .take(1)
            .next()
            .map(|(k, _)| unsafe { k.as_ref() })
    }
}

#[cfg(test)]
mod tests {
    use super::{InternId, Interner, SymbolId};

    #[test]
    fn it_works() {
        let mut interner = Interner::new();

        assert_eq!(interner.intern("hello"), SymbolId::id(0));
        assert_eq!(interner.intern("world"), SymbolId::id(1));
        assert_eq!(interner.get(&SymbolId::id(0)), Some(&"hello"));
    }
}
