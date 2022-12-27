use ast::prelude::{
    Const, Function, FunctionParam, ParamKind, Spanned, Statement, Token, Type, TypeAlias,
};

use super::Parser;

impl<'a> Parser<'a> {
    ///  type -> identifier | "[" type (";" number)? "]" | "fn" "(" type* ")" ("->" type )? | ()
    pub(crate) fn parse_type(&mut self) -> Spanned<Type> {
        if self.match_token(Token::LeftBracket) {
            let start = self.prev.span();
            let ty = self.parse_type();
            let mut length = None;

            if self.check(Token::SemiColon) {
                self.advance();

                self.consume(Token::Number, "Expected an array length after `;`");

                length = self.src[self.prev.span().start.absolute..self.prev.span().end.absolute]
                    .parse::<usize>()
                    .ok();
            }

            let end = self.consume_get_span(Token::RightBracket, "Expected `]`");
            Spanned::new(
                Type::Array {
                    ty: Box::new(ty),
                    length,
                },
                start.merge(end),
            )
        } else if self.match_token(Token::Fun) {
            let start = self.prev.span();

            self.consume(Token::LeftParen, "Expected `(` ");

            let mut params = Vec::new();

            if !self.check(Token::RightParen) {
                loop {
                    let ty = self.parse_type();

                    params.push(ty);

                    if self.check(Token::Comma) {
                        self.advance();
                    } else {
                        break;
                    }
                }
            }

            self.consume(Token::RightParen, "Expected `)`");

            let mut returns = None;

            if self.match_token(Token::FunctionReturn) {
                returns = Some(Box::new(self.parse_type()))
            }

            let end = self.prev.span();

            Spanned::new(Type::Function { params, returns }, start.merge(end))
        } else if self.match_token(Token::Identifier) {
            let id = self
                .symbols
                .intern(&self.src[self.prev.span().start.absolute..self.prev.span().end.absolute]);

            Spanned::new(Type::Identifier(id), self.prev.span())
        } else {
            self.advance();
            Spanned::new(Type::Error, self.prev.span())
        }
    }

    pub(crate) fn type_alias(&mut self) -> Spanned<TypeAlias> {
        let start = self.prev.span();
        self.consume(Token::Identifier, "Expected variable name");
        let id = Spanned::new(
            self.symbols
                .intern(&self.src[self.prev.span().start.absolute..self.prev.span().end.absolute]),
            self.prev.span(),
        );

        self.consume(Token::Equal, "Expected `=`");

        let ty = self.parse_type();

        let end = ty.span();

        Spanned::new(TypeAlias { name: id, ty }, start.merge(end))
    }

    pub(crate) fn const_declaration(&mut self) -> Spanned<Const> {
        let start = self.prev.span();
        self.consume(Token::Identifier, "Expected variable name");
        let id = Spanned::new(
            self.symbols
                .intern(&self.src[self.prev.span().start.absolute..self.prev.span().end.absolute]),
            self.prev.span(),
        );

        let mut ty = None;

        if self.match_token(Token::Colon) {
            ty = Some(self.parse_type());
        }

        self.consume(Token::Assignment, "Expected `:=`");

        let initializer = self.expression();

        let end = self.consume_get_span(Token::SemiColon, "Expected `;` after a const declaration");

        Spanned::new(
            Const {
                name: id,
                ty,
                initializer,
            },
            start.merge(end),
        )
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

    pub(crate) fn parse_params(&mut self, kind: ParamKind) -> Vec<Spanned<FunctionParam>> {
        let mut params = Vec::with_capacity(32);

        if !self.check(Token::RightParen) && !self.check(Token::Bar) {
            loop {
                if params.len() >= 32 {
                    self.error("Too many params");
                    break;
                };

                self.consume(
                    Token::Identifier,
                    &format!("Expected a {} identifier", kind),
                );

                let start = self.prev.span();

                let id = self
                    .symbols
                    .intern(&self.src[start.start.absolute..start.end.absolute]);

                self.consume(Token::Colon, "Expected `:`");

                let ty = self.parse_type();

                let end = ty.span();

                params.push(Spanned::new(
                    FunctionParam { name: id, ty },
                    start.merge(end),
                ));

                if self.check(Token::Comma) {
                    self.advance();
                } else {
                    break;
                }
            }
        }

        params
    }

    pub(crate) fn fn_declaration(&mut self) -> Spanned<Function> {
        let start = self.prev.span();
        self.consume(Token::Identifier, "Expected variable name");
        let id = Spanned::new(
            self.symbols
                .intern(&self.src[self.prev.span().start.absolute..self.prev.span().end.absolute]),
            self.prev.span(),
        );

        let end = self.prev.span();

        self.consume(Token::LeftParen, "Expected '(' ");

        let params = self.parse_params(ParamKind::Function);

        self.consume(Token::RightParen, "Expected ')'");

        let mut returns = None;

        if self.match_token(Token::FunctionReturn) {
            returns = Some(self.parse_type());
        }

        self.consume(Token::LeftBrace, "Expected `{`");

        let body = self.block();

        Spanned::new(
            Function {
                name: id,
                params,
                body,
                returns,
            },
            start.merge(end),
        )
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

        if self.match_token(Token::Assignment) {
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
