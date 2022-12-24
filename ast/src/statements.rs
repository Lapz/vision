use crate::{expression::Expression, span::Spanned};

#[derive(Debug)]
pub enum Statement {
    Expression(Spanned<Expression>),
}
