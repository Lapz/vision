use crate::intern::{LiteralId, SymbolId};

pub enum Expression {
    Literal(LiteralId),
    Ternary {
        cond: Box<Expression>,
        lhs: Box<Expression>,
        rhs: Box<Expression>,
    },
    Identifier(SymbolId),
    Binary {
        op: BinaryOp,
        lhs: Box<Expression>,
        rhs: Box<Expression>,
    },
    Grouping(Box<Expression>),
    Call {
        callee: SymbolId,
        args: Vec<Expression>,
    },
    Unary {
        op: UnaryOp,
        rhs: Box<Expression>,
    },
}

pub enum Literal {
    String,
    Number,
    Bool(bool),
    Nil,
}

pub enum BinaryOp {
    Plus,
    Minus,
    Slash,
    Star,
}
pub enum UnaryOp {
    Bang,
    Plus,
    Minus,
}
