use crate::{expression::Expression, span::Spanned};

pub enum Statement {
    Expression(Spanned<Expression>),
}
