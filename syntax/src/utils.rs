use ast::prelude::{BinaryOp, Expression, Span, Spanned, Token, UnaryOp};

use super::parser::{ParseRule, Parser, Precedence};

#[macro_export]
macro_rules! matches {
    ($scanner:ident,$char:literal,$lhs:path,$rhs:path) => {{
        if $scanner.matches($char) {
            $scanner.make_token($lhs)
        } else {
            $scanner.make_token($rhs)
        }
    }};
}

#[macro_export]
macro_rules! hashmap {
    (@single $($x:tt)*) => (());
    (@count $($rest:expr),*) => (<[()]>::len(&[$(hashmap!(@single $rest)),*]));

    ($($key:expr => $value:expr,)+) => { hashmap!($($key => $value),+) };
    ($($key:expr => $value:expr),*) => {
        {
            let _cap = hashmap!(@count $($key),*);
            let mut _map = ::std::collections::HashMap::with_capacity(_cap);
            $(
                let _ = _map.insert($key, $value);
            )*
            _map
        }
    };
}

impl<'a> Parser<'a> {
    pub(crate) fn get_rule(&self, ty: Token) -> ParseRule<'a> {
        self.rules[&ty]
    }

    pub(crate) fn error(&mut self, msg: &str) -> Spanned<Expression> {
        println!("{}", msg);

        self.panic_mode = true;
        self.had_error = true;
        Spanned::new(Expression::Error, self.prev.span())
    }

    pub fn match_token(&mut self, ty: Token) -> bool {
        if !self.check(ty) {
            return false;
        }
        self.advance();

        true
    }

    pub(crate) fn consume(&mut self, ty: Token, arg: &str) {
        if *self.current.value() == ty {
            self.advance();
            return;
        }

        // @TODO error
        // self.error_at_current(arg);
    }

    pub(crate) fn consume_get_span(&mut self, ty: Token, arg: &str) -> Span {
        if *self.current.value() == ty {
            self.advance();

            self.prev.span()
        } else {
            // @TODO error
            // self.error_at_current(arg);
            self.current.span()
        }
    }

    pub(crate) fn check(&self, ty: Token) -> bool {
        self.current.value() == &ty
    }

    pub(crate) fn parse_with_precedence(&mut self, precedence: Precedence) -> Spanned<Expression> {
        self.advance();

        let prefix_rule = self.get_rule(*self.prev.value()).prefix;

        let mut expr = match prefix_rule {
            Some(prefix_rule) => prefix_rule(self),
            None => {
                // @TODO error
                self.error("Expect expression.")
            }
        };

        while precedence <= self.get_rule(*self.current.value()).precedence {
            self.advance();

            let infix_rule = self.get_rule(*self.prev.value()).infix;

            expr = match infix_rule {
                Some(infix_rule) => infix_rule(self, expr),
                None => {
                    // @TODO error
                    self.error("Expect expression.")
                }
            };
        }

        expr
    }

    pub(crate) fn get_unary_op(&mut self) -> Spanned<UnaryOp> {
        let op = match *self.prev.value() {
            Token::Bang => UnaryOp::Bang,
            Token::Plus => UnaryOp::Plus,
            Token::Minus => UnaryOp::Minus,
            _ => unreachable!(),
        };

        Spanned::new(op, self.prev.span())
    }

    pub(crate) fn get_binary_op(&mut self) -> Spanned<BinaryOp> {
        let op = match *self.prev.value() {
            Token::BangEqual => BinaryOp::BangEqual,
            Token::EqualEqual => BinaryOp::EqualEqual,
            Token::Greater => BinaryOp::Greater,
            Token::GreaterEqual => BinaryOp::GreaterEqual,
            Token::Less => BinaryOp::Less,
            Token::LessEqual => BinaryOp::LessEqual,
            Token::Plus => BinaryOp::Plus,
            Token::Minus => BinaryOp::Minus,
            Token::Star => BinaryOp::Star,
            Token::Slash => BinaryOp::Slash,
            Token::Equal => BinaryOp::Equal,
            _ => unreachable!(),
        };

        Spanned::new(op, self.prev.span())
    }
}
