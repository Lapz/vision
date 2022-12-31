use super::{parser::Precedence, Parser};
use ast::prelude::{Expression, Literal, Spanned, Statement, Token};

impl<'a> Parser<'a> {
    pub(crate) fn expression_statement(&mut self) -> Spanned<Statement> {
        let expr = self.expression();

        let end = self.consume_get_span(Token::SemiColon, "Expected ';' after expression.");

        let span = expr.span();

        Spanned::new(Statement::Expression(expr), span.merge(end))
    }

    pub(crate) fn expression(&mut self) -> Spanned<Expression> {
        self.parse_with_precedence(Precedence::Assignment)
    }

    pub(crate) fn unary(&mut self) -> Spanned<Expression> {
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

    pub(crate) fn identifier(&mut self) -> Spanned<Expression> {
        let id = self.get_identifier();

        Spanned::new(Expression::Identifier(id), id.span())
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

    pub(crate) fn call(&mut self, lhs: Spanned<Expression>) -> Spanned<Expression> {
        let mut args = Vec::new();

        let mut count = 0;

        if !self.check(Token::RightParen) {
            loop {
                args.push(self.expression());

                if count == 255 {
                    self.error("Can't have more than 255 arguments.");
                }

                count += 1;

                if !self.match_token(Token::Comma) {
                    break;
                }
            }
        }

        let start = lhs.span();

        let end = self.consume_get_span(Token::RightParen, "Expected ')' after arguments");

        Spanned::new(
            Expression::Call {
                callee: Box::new(lhs),
                args: args,
            },
            start.merge(end),
        )
    }

    pub(crate) fn grouping(&mut self) -> Spanned<Expression> {
        let expr = self.expression();

        self.consume(Token::RightParen, "Expect ')' after expression.");

        let start = expr.span();
        let end = self.consume_get_span(Token::SemiColon, "Expected ';' after expression.");
        Spanned::new(Expression::Grouping(Box::new(expr)), start.merge(end))
    }

    pub(crate) fn literal(&mut self) -> Spanned<Expression> {
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
