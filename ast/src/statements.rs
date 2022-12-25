use crate::{
    expression::Expression,
    intern::SymbolId,
    prelude::{Span, Type},
    span::Spanned,
};
use std::fmt::{self, Display};
#[derive(Debug)]
pub enum Statement {
    Expression(Spanned<Expression>),
    While {
        cond: Spanned<Expression>,
        body: Box<Spanned<Statement>>,
    },
    If {
        cond: Spanned<Expression>,
        then: Box<Spanned<Statement>>,
        else_: Option<Box<Spanned<Statement>>>,
    },
    Block(Vec<Spanned<Statement>>),
    Return(Option<Spanned<Expression>>),
    Break,
    Continue,
    Let {
        identifier: SymbolId,
        ty: Option<Spanned<Type>>,
        init: Option<Spanned<Expression>>,
    },
}

impl Display for Spanned<Statement> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value())
    }
}

impl Display for Statement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Statement::Expression(expr) => write!(f, "{};", expr),
            Statement::While { cond, body } => todo!(),
            Statement::Return(_) => todo!(),
            Statement::Block(_) => todo!(),
            Statement::If { cond, then, else_ } => todo!(),
            Statement::Break => write!(f, "break"),
            Statement::Continue => write!(f, "continue"),
            Statement::Let {
                identifier,
                ty,
                init,
            } => {
                write!(f, "let {} := ", identifier)?;
                match init {
                    Some(expr) => write!(f, "{}", &expr),
                    None => write!(f, "nil"),
                }
            }
        }
    }
}
