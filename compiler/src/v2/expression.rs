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

    pub(crate) fn identifier(&mut self, _can_assign: bool) -> Spanned<Expression> {
        let span = self.prev.span();
        let id = self
            .symbols
            .intern(&self.src[span.start.absolute..span.end.absolute]);

        Spanned::new(Expression::Identifier(id), span)
    }

    pub(crate) fn ternary(&mut self, cond: Spanned<Expression>) -> Spanned<Expression> {
        let lhs = self.expression();

        self.consume(Token::Colon, "Expect ':' after ternary expression");

        let rhs = self.expression();

        let start = cond.span();
        let end = rhs.span();

        Spanned::new(
            Expression::Ternary {
                cond: Box::new(cond),
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            },
            start.merge(end),
        )
    }

    pub(crate) fn binary(&mut self, lhs: Spanned<Expression>) -> Spanned<Expression> {
        let op = self.get_binary_op();

        let rule = self.get_rule(*self.prev.value());
        let expr = self.parse_with_precedence(rule.precedence.higher());

        let start = lhs.span();

        let end = expr.span();
        Spanned::new(
            Expression::Binary {
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(expr),
            },
            start.merge(end),
        )
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
