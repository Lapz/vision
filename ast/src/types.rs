use crate::{intern::SymbolId, prelude::Spanned};
#[derive(Debug)]
pub enum Type {
    /// number, float, string etc
    Identifier(SymbolId),
    /// [number;2] or  [numbe]
    Array {
        ty: Box<Spanned<Type>>,
        length: Option<usize>,
    },
    /// Function
    /// fn(i32,i32) -> bool
    Function {
        params: Vec<Spanned<Type>>,
        returns: Option<Box<Spanned<Type>>>,
    },
    Void,
    Error,
}
