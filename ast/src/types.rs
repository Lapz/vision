use crate::{intern::SymbolId, prelude::Spanned};
#[derive(Debug, Clone)]
pub enum Type {
    /// number, float, string etc
    Identifier(Spanned<SymbolId>),
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
