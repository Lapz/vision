use crate::{
    intern::{LiteralId, SymbolId},
    prelude::Spanned,
};
#[derive(Debug)]
pub enum Expression {
    Literal(Literal),
    Ternary {
        cond: Box<Spanned<Expression>>,
        lhs: Box<Spanned<Expression>>,
        rhs: Box<Spanned<Expression>>,
    },
    Identifier(SymbolId),
    Binary {
        op: BinaryOp,
        lhs: Box<Spanned<Expression>>,
        rhs: Box<Spanned<Expression>>,
    },
    Grouping(Box<Spanned<Expression>>),
    Call {
        callee: SymbolId,
        args: Vec<Spanned<Expression>>,
    },
    Unary {
        op: Spanned<UnaryOp>,
        rhs: Box<Spanned<Expression>>,
    },
    Error,
}

#[derive(Debug)]
pub enum Literal {
    String,
    Number,
    Bool(bool),
    Nil,
}
#[derive(Debug)]
pub enum BinaryOp {
    Plus,
    Minus,
    Slash,
    Star,
}
#[derive(Debug)]
pub enum UnaryOp {
    Bang,
    Plus,
    Minus,
}
