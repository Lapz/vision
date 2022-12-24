use crate::intern::SymbolId;

pub enum Type {
    /// number, float, string etc
    Identifier(SymbolId),
    /// number[] or  number[2]
    Array {
        ty: Box<Type>,
        length: Option<usize>,
    },
    /// Function
    /// fn(i32,i32) -> bool
    Function {
        params: Vec<Type>,
    },
    Void,
}
