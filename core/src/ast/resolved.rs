use std::fmt::{self, Display};

use ast::prelude::{Literal, Spanned, SymbolId};

#[derive(Debug)]
pub enum Expression {
    Literal(Literal),
    Ternary {
        cond: Box<Spanned<Expression>>,
        lhs: Box<Spanned<Expression>>,
        rhs: Box<Spanned<Expression>>,
    },
    Identifier(Spanned<SymbolId>),
    Binary {
        op: Spanned<BinaryOp>,
        lhs: Box<Spanned<Expression>>,
        rhs: Box<Spanned<Expression>>,
    },
    Grouping(Box<Spanned<Expression>>),
    Call {
        callee: Box<Spanned<Expression>>,
        args: Vec<Spanned<Expression>>,
    },
    Unary {
        op: Spanned<UnaryOp>,
        rhs: Box<Spanned<Expression>>,
    },
    Error,
}

#[derive(Debug)]
pub enum BinaryOp {
    Plus,
    Minus,
    Slash,
    Star,
    EqualEqual,
    Greater,
    Less,
    Assignment,
}
#[derive(Debug)]
pub enum UnaryOp {
    Bang,
    Plus,
    Minus,
}

#[derive(Debug, Clone)]
pub enum Type {
    /// number, float, string etc
    Named(Spanned<SymbolId>),
    /// [number;2] or  [numbe]
    Array {
        ty: Box<Spanned<Type>>,
        length: Option<usize>,
    },
    /// Function
    /// fn(i32,i32) -> bool
    Function {
        params: Vec<Spanned<Type>>,
        returns: Box<Spanned<Type>>,
    },
    Void,
    Error,
}
