use crate::{expression::Expression, span::Spanned};
use std::fmt::{self, Display};
#[derive(Debug)]
pub enum Statement {
    Expression(Spanned<Expression>),
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
        }
    }
}
