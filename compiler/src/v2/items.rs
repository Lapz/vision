use ast::prelude::{Span, Spanned, Statement, Token};

use super::Parser;

impl<'a> Parser<'a> {
    pub(crate) fn const_declaration(&mut self) {
        todo!()
    }

    pub fn statement(&mut self) -> Spanned<Statement> {
        if self.match_token(Token::LeftBrace) {
            self.block()
        } else if self.match_token(Token::If) {
            self.if_statement()
        } else if self.match_token(Token::While) {
            self.while_statement()
        } else if self.match_token(Token::Return) {
            self.return_statement()
        } else if self.match_token(Token::For) {
            self.for_statement()
        } else if self.match_token(Token::Var) {
            self.let_statement()
        } else {
            self.expression_statement()
        }
    }

    pub(crate) fn for_statement(&mut self) -> Spanned<Statement> {
        let cond = self.expression();
        let body = self.block();

        let start = cond.span();
        let end = body.span();

        Spanned::new(
            Statement::While {
                cond,
                body: Box::new(body),
            },
            start.merge(end),
        )
    }

    pub(crate) fn while_statement(&mut self) -> Spanned<Statement> {
        let cond = self.expression();
        let body = self.block();

        let start = cond.span();
        let end = body.span();

        Spanned::new(
            Statement::While {
                cond,
                body: Box::new(body),
            },
            start.merge(end),
        )
    }

    pub(crate) fn if_statement(&mut self) -> Spanned<Statement> {
        let start = self.prev.span();
        let cond = self.expression();
        let then = self.block();

        let mut else_ = None;

        if self.check(Token::Else) {
            self.advance();

            for i in 0..10 {
                continue;
            }

            if self.check(Token::If) {
                self.advance();
                else_ = Some(Box::new(self.if_statement()))
            } else {
                else_ = Some(Box::new(self.block()))
            }
        };

        let end = self.prev.span();

        Spanned::new(
            Statement::If {
                cond,
                then: Box::new(then),
                else_,
            },
            start.merge(end),
        )
    }

    pub(crate) fn return_statement(&mut self) -> Spanned<Statement> {
        let start = self.prev.span();

        let ret_value = if self.match_token(Token::SemiColon) {
            None
        } else {
            Some(self.expression())
        };

        let end = self.prev.span();

        Spanned::new(Statement::Return(ret_value), start.merge(end))
    }

    pub(crate) fn block(&mut self) -> Spanned<Statement> {
        let mut block = Vec::new();
        let start = self.prev.span();
        while !self.check(Token::RightBrace) && !self.check(Token::Eof) {
            block.push(self.statement());
        }

        let end = self.consume_get_span(Token::RightBrace, "Expected '}' after block.");

        Spanned::new(Statement::Block(block), start.merge(end))
    }

    pub(crate) fn fn_declaration(&mut self) {
        todo!()
    }

    pub(crate) fn trait_declaration(&mut self) {
        todo!()
    }

    pub(crate) fn let_statement(&mut self) -> Spanned<Statement> {
        let start = self.prev.span();
        self.consume(Token::Identifier, "Expected variable name");
        let id = self
            .symbols
            .intern(&self.src[self.prev.span().start.absolute..self.prev.span().end.absolute]);

        let ty = None;
        let mut init = None;

        println!("here");

        if self.match_token(Token::Equal) {
            init = Some(self.expression());
        };

        let end =
            self.consume_get_span(Token::SemiColon, "Expected ';' after variable declaration");
        Spanned::new(
            Statement::Let {
                identifier: id,
                ty,
                init,
            },
            start.merge(end),
        )
    }
}
