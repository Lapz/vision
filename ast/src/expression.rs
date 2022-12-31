use std::fmt::{self, Display};

use crate::{intern::SymbolId, prelude::Spanned};
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
    BangEqual,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    Assignment,
}
#[derive(Debug)]
pub enum UnaryOp {
    Bang,
    Plus,
    Minus,
}

impl Display for Spanned<Expression> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value())
    }
}
impl Display for Spanned<UnaryOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value())
    }
}

impl Display for Spanned<BinaryOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value())
    }
}
impl Display for UnaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UnaryOp::Bang => write!(f, "!"),
            UnaryOp::Plus => write!(f, "+"),
            UnaryOp::Minus => write!(f, "-"),
        }
    }
}
impl Display for BinaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BinaryOp::Plus => write!(f, "+"),
            BinaryOp::Minus => write!(f, "-"),
            BinaryOp::Slash => write!(f, "/"),
            BinaryOp::Star => write!(f, "*"),
            BinaryOp::Assignment => write!(f, ":="),
            BinaryOp::BangEqual => write!(f, "!="),
            BinaryOp::EqualEqual => write!(f, "=="),
            BinaryOp::Greater => write!(f, ">"),
            BinaryOp::GreaterEqual => write!(f, ">="),
            BinaryOp::Less => write!(f, "<"),
            BinaryOp::LessEqual => write!(f, "<="),
        }
    }
}

impl Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expression::Literal(lit) => match lit {
                Literal::String => write!(f, "string"),
                Literal::Number => write!(f, "number"),
                Literal::Bool(b) => write!(f, "{}", b),
                Literal::Nil => {
                    write!(f, "nil")
                }
            },
            Expression::Ternary { cond, lhs, rhs } => {
                write!(f, "{} ? {} : {}", cond, lhs, rhs)
            }
            Expression::Identifier(ident) => write!(f, "{}", ident.value()),
            Expression::Binary { op, lhs, rhs } => write!(f, "{} {} {}", lhs, op, rhs),
            Expression::Grouping(expr) => write!(f, "({})", expr),
            Expression::Call { callee, args } => todo!(),
            Expression::Unary { op, rhs } => write!(f, "{}{}", op, rhs),
            Expression::Error => write!(f, "error"),
        }
    }
}
