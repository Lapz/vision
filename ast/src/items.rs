use crate::{intern::SymbolId, span::Spanned, statements::Statement, types::Type};

pub struct Program {
    functions: Vec<Function>,
}

pub struct Function {
    name: SymbolId,
    params: Vec<FunctionParam>,
    body: Spanned<Statement>,
    returns: Type,
}

pub struct FunctionParam {
    pub name: SymbolId,
    pub ty: Type,
}

pub struct Trait {}

pub struct Record {}
