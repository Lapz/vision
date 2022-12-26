use std::fmt::Display;

use crate::{
    expression::Expression, intern::SymbolId, span::Spanned, statements::Statement, types::Type,
};
#[derive(Debug)]
pub struct Program {
    functions: Vec<Spanned<Function>>,
    consts: Vec<Spanned<Const>>,
    type_alias: Vec<Spanned<TypeAlias>>,
}
#[derive(Debug)]
pub struct Function {
    pub name: SymbolId,
    pub params: Vec<Spanned<FunctionParam>>,
    pub body: Spanned<Statement>,
    pub returns: Option<Spanned<Type>>,
}
#[derive(Debug, Clone, Copy)]
pub enum ParamKind {
    Function,
    Closure,
}
#[derive(Debug)]
pub struct FunctionParam {
    pub name: SymbolId,
    pub ty: Spanned<Type>,
}

#[derive(Debug)]

pub struct Trait {}
#[derive(Debug)]
pub struct Const {
    pub name: SymbolId,
    pub ty: Option<Spanned<Type>>,
    pub initializer: Spanned<Expression>,
}
#[derive(Debug)]
pub struct Struct {}
#[derive(Debug)]
pub struct TypeAlias {
    pub name: SymbolId,
    pub ty: Spanned<Type>,
}

impl Program {
    pub fn new() -> Self {
        Self {
            functions: Vec::new(),
            consts: Vec::new(),
            type_alias: Vec::new(),
        }
    }

    pub fn add_const(&mut self, const_: Spanned<Const>) {
        self.consts.push(const_)
    }

    pub fn add_fn(&mut self, fun: Spanned<Function>) {
        self.functions.push(fun)
    }

    pub fn add_type_alias(&mut self, alias: Spanned<TypeAlias>) {
        self.type_alias.push(alias)
    }
}

impl Display for ParamKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParamKind::Function => write!(f, "function"),
            ParamKind::Closure => write!(f, "closure"),
        }
    }
}
