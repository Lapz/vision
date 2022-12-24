use super::{parser::Precedence, Parser};
use ast::prelude::{Expression, Literal, Spanned, Statement, Token};

impl<'a> Parser<'a> {
    pub fn expression_statement(&mut self) -> Spanned<Statement> {
        let expr = self.expression();

        let end = self.consume_get_span(Token::SemiColon, "Expected ';' after expression.");

        let span = expr.span();

        Spanned::new(Statement::Expression(expr), span.merge(end))
    }

    pub(crate) fn expression(&mut self) -> Spanned<Expression> {
        self.parse_with_precedence(Precedence::Assignment)
    }

    pub(crate) fn unary(&mut self, _can_assign: bool) -> Spanned<Expression> {
        let op = self.get_unary_op();

        let rhs = self.parse_with_precedence(Precedence::Unary);

        let start = op.span();
        let end = rhs.span();

        Spanned::new(
            Expression::Unary {
                op,
                rhs: Box::new(rhs),
            },
            start.merge(end),
        )
    }

    pub(crate) fn ternary(&mut self, _can_assign: bool) -> Spanned<Expression> {
        todo!()
    }

    pub(crate) fn binary(&mut self) -> Spanned<Expression> {
        let op = self.get_binary_op();

        todo!()
    }

    pub(crate) fn grouping(&mut self, _can_assign: bool) -> Spanned<Expression> {
        let expr = self.expression();

        self.consume(Token::RightParen, "Expect ')' after expression.");

        let start = expr.span();
        let end = self.consume_get_span(Token::SemiColon, "Expected ';' after expression.");

        Spanned::new(Expression::Grouping(Box::new(expr)), start.merge(end))
    }

    pub(crate) fn literal(&mut self, _can_assign: bool) -> Spanned<Expression> {
        let literal = match *self.prev.value() {
            Token::Number => Literal::Number,
            Token::True => Literal::Bool(true),
            Token::False => Literal::Bool(false),
            Token::Nil => Literal::Nil,
            Token::String => Literal::String,
            _ => unreachable!(),
        };
        Spanned::new(Expression::Literal(literal), self.prev.span())
    }
}
