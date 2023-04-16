use std::{
    borrow::Borrow,
    collections::HashMap,
    fmt::{Debug, Display},
    hash::Hash,
    marker::PhantomData,
    ops::Deref,
    ptr::NonNull,
};

pub type SymbolDB = Interner<SymbolId>;

#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq)]
pub struct SymbolId(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LiteralId(u32);

impl Display for SymbolId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "${}", self.0)
    }
}

pub trait InternId: Copy + Clone + Debug {
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

impl<T: ?Sized + Debug> Debug for OwnedPtr<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OwnedPtr")
            .field("ptr", &self.ptr)
            .field("marker", &self.marker)
            .finish()
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
pub struct Interner<T: InternId> {
    map: HashMap<&'static str, T>,
    strings: Vec<&'static str>,
    buf: String,
    full: Vec<String>,
}
impl<T: InternId> Debug for Interner<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Interner")
            .field("map", &self.map)
            .field("strings", &self.strings)
            .field("buf", &self.buf)
            .field("full", &self.full)
            .finish()
    }
}

impl Default for SymbolDB {
    fn default() -> Self {
        let mut db = Self {
            map: HashMap::with_capacity(8),
            strings: Vec::with_capacity(8),
            buf: String::with_capacity(8),
            full: Vec::with_capacity(8),
        };

        db.intern("number");
        db.intern("float");
        db.intern("bool");
        db.intern("string");

        db
    }
}

impl<T: InternId> Interner<T> {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            strings: Vec::with_capacity(8),
            buf: String::with_capacity(8),
            full: Vec::with_capacity(8),
        }
    }

    pub fn new_with<const N: usize>(symbols: [&str; N]) -> Self {
        let mut db = Self {
            map: HashMap::new(),
            strings: Vec::with_capacity(N),
            buf: String::with_capacity(N),
            full: Vec::with_capacity(N),
        };

        for i in symbols {
            db.intern(i);
        }

        db
    }

    pub fn intern(&mut self, item: &str) -> T {
        let borrowed = item.borrow();

        if let Some((_, id)) = self.map.get_key_value(borrowed) {
            return *id;
        }

        let string: &'static str = unsafe { self.alloc(item) };

        let id = T::id(self.map.borrow().len() as u32);

        self.map.insert(string, id);
        self.strings.push(string);

        id
    }

    unsafe fn alloc(&mut self, name: &str) -> &'static str {
        let cap = self.buf.capacity();
        if cap < self.buf.len() + name.len() {
            let new_cap = (cap.max(name.len()) + 1).next_power_of_two();
            let new_buf = String::with_capacity(new_cap);
            let old_buf = std::mem::replace(&mut self.buf, new_buf);
            self.full.push(old_buf);
        }
        let interned = {
            let start = self.buf.len();
            self.buf.push_str(name);
            &self.buf[start..]
        };
        &*(interned as *const str)
    }

    pub fn lookup(&self, key: &T) -> &'static str {
        self.strings[key.index() as usize]
    }
}

pub const DEFAULT_TYPES: [&'static str; 4] = ["number", "string", "boolean", "float"];

#[cfg(test)]
mod tests {
    use super::{InternId, Interner, SymbolId};

    #[test]
    fn it_works() {
        let mut interner = Interner::new();

        assert_eq!(interner.intern("hello"), SymbolId::id(0));
        assert_eq!(interner.intern("world"), SymbolId::id(1));
        assert_eq!(interner.intern("hello"), SymbolId::id(0));
        assert_eq!(interner.lookup(&SymbolId::id(0)), "hello");
    }
}
