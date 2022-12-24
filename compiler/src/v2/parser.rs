use std::collections::HashMap;

use crate::hashmap;

use super::lexer::Lexer;
use ast::prelude::{Expression, Interner, LiteralId, Position, Span, Spanned, SymbolId, Token};
pub struct Parser<'a> {
    pub(crate) src: &'a str,
    pub(crate) lexer: Lexer<'a>,
    pub(crate) had_error: bool,
    pub(crate) panic_mode: bool,
    pub(crate) current: Spanned<Token>,
    pub(crate) prev: Spanned<Token>,
    pub(crate) rules: HashMap<Token, ParseRule<'a>>,
    pub(crate) symbols: Interner<&'a str, SymbolId>,
}

#[derive(Clone, Copy)]
pub struct ParseRule<'a> {
    pub(crate) prefix: Option<fn(&mut Parser<'a>, bool) -> Spanned<Expression>>,
    pub(crate) infix: Option<fn(&mut Parser<'a>, Spanned<Expression>) -> Spanned<Expression>>,
    pub(crate) precedence: Precedence,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum Precedence {
    None,
    Assignment, // =
    Or,         // or
    And,        // and
    Equality,   // == !=
    Comparison, // < > <= >=
    Term,       // + -
    Factor,     // * /
    Unary,      // ! -
    Call,       // . ()
    Primary,
}

impl<'a> Parser<'a> {
    pub fn new(src: &'a str) -> Parser {
        let mut parser = Parser {
            src,
            lexer: Lexer::new(src),
            had_error: false,
            panic_mode: false,
            prev: Spanned::new(
                Token::Eof,
                Span::new(Position::new(1, 1, 0), Position::new(1, 1, 0)),
            ),
            current: Spanned::new(
                Token::Eof,
                Span::new(Position::new(1, 1, 0), Position::new(1, 1, 0)),
            ),
            symbols: Interner::new(),
            rules: hashmap! {
                Token::LeftParen => ParseRule {
                        prefix: Some(Parser::grouping),
                        infix: None,
                        precedence: Precedence::Call,
                },
                Token::RightParen => ParseRule::default(),
                Token::True => ParseRule {
                        prefix: Some(Parser::literal),
                        infix: None,
                        precedence: Precedence::None,
                },
                Token::False => ParseRule {
                        prefix: Some(Parser::literal),
                        infix: None,
                        precedence: Precedence::None,
                },
                Token::Nil => ParseRule {
                        prefix: Some(Parser::literal),
                        infix: None,
                        precedence: Precedence::None,
                },
                Token::String => ParseRule {
                        prefix: Some(Parser::literal),
                        infix: None,
                        precedence: Precedence::None,
                },
                Token::Number => ParseRule {
                        prefix: Some(Parser::literal),
                        infix: None,
                        precedence: Precedence::None,
                },
                Token::SemiColon => ParseRule::default(),
                Token::Minus=> ParseRule {
                        prefix: Some(Parser::unary),
                        infix:Some(Parser::binary),
                        precedence: Precedence::Term,
                 },
                Token::Plus => ParseRule {
                    prefix: Some(Parser::unary),
                    infix: Some(Parser::binary),
                    precedence: Precedence::Term,
                },
                Token::Slash => ParseRule {
                    prefix: None,
                    infix: None, // Some(Parser::binary),
                    precedence: Precedence::Factor,
                },
                Token::Star => ParseRule {
                    prefix: None,
                    infix: Some(Parser::binary),
                    precedence: Precedence::Factor,
                },
                Token::Bang => ParseRule {
                    prefix: Some(Parser::unary),
                    infix: None,
                    precedence: Precedence::None
                },
                Token::BangEqual => ParseRule {
                        prefix: None,
                        infix: Some(Parser::binary),
                        precedence: Precedence::Equality,
                },
                Token::Equal => ParseRule::default(),
                Token::EqualEqual => ParseRule {
                    prefix: None,
                    infix: Some(Parser::binary),
                    precedence: Precedence::Equality,
                },
                Token::Greater => ParseRule {
                    prefix: None,
                    infix: Some(Parser::binary),
                    precedence: Precedence::Comparison,
                },
                Token::GreaterEqual => ParseRule {
                    prefix: None,
                    infix: Some(Parser::binary),
                    precedence: Precedence::Comparison,
                },
                Token::Less => ParseRule {
                    prefix: None,
                    infix: Some(Parser::binary),
                    precedence: Precedence::Comparison,
                },
                Token::LessEqual => ParseRule {
                    prefix: None,
                    infix: Some(Parser::binary),
                    precedence: Precedence::Comparison,
                },
                Token::And => ParseRule {
                    prefix: None,
                    infix: Some(Parser::binary),
                    precedence: Precedence::And,
                },
                Token::Or =>  ParseRule {
                    prefix: None,
                    infix: Some(Parser::binary),
                    precedence: Precedence::Or,
                },
                Token::QuestionMark => ParseRule {
                    prefix: None,
                    infix: Some(Parser::ternary),
                    precedence: Precedence::Assignment
                },
                Token::Identifier => ParseRule {
                    prefix: Some(Parser::identifier),
                    infix: None,
                    precedence: Precedence::None,
                },
                Token::Eof => ParseRule::default(),
                Token::Var => ParseRule::default(),
                Token::While => ParseRule::default(),
                Token::Error => ParseRule::default(),
                Token::Print => ParseRule::default(),
                Token::Return => ParseRule::default(),
                Token::Super => ParseRule::default(),
                Token::This => ParseRule::default(),
                Token::For => ParseRule::default(),
                Token::Fun => ParseRule::default(),
                Token::If => ParseRule::default(),
                Token::Class => ParseRule::default(),
                Token::Else => ParseRule::default(),
                Token::Equal => ParseRule {
                    prefix: None,
                    infix: Some(Parser::binary),
                    precedence: Precedence::Assignment,
                },
                Token::LeftBrace => ParseRule::default(),
                Token::RightBrace => ParseRule::default(),
                Token::Comma => ParseRule::default(),
                Token::Dot => ParseRule::default(),
            },
        };

        parser.advance();

        parser
    }

    pub fn advance(&mut self) {
        std::mem::swap(&mut self.prev, &mut self.current);

        loop {
            self.current = self.lexer.next_token();

            if self.current.value() != &Token::Error {
                break;
            }

            // @TODO error reporting
        }
    }

    pub fn declaration(&mut self) {
        if self.match_token(Token::Const) {
            self.const_declaration()
        } else if self.match_token(Token::Fun) {
            self.fn_declaration()
        } else if self.match_token(Token::Trait) {
            self.trait_declaration()
        } else {
            self.statement();
        }

        if self.panic_mode {
            self.synchronize();
        }
    }

    fn synchronize(&mut self) {
        self.panic_mode = false;

        while self.current.value() != &Token::Eof {
            if self.prev.value() == &Token::SemiColon {
                return;
            }

            match *self.current.value() {
                Token::Class
                | Token::Fun
                | Token::Var
                | Token::For
                | Token::If
                | Token::While
                | Token::Print
                | Token::Return => return,
                _ => {}
            }

            self.advance();
        }
    }
}

impl Precedence {
    pub(crate) fn higher(&self) -> Precedence {
        match *self {
            Precedence::None | Precedence::Assignment => Precedence::Or,
            Precedence::Or => Precedence::And,
            Precedence::And => Precedence::Equality,
            Precedence::Equality => Precedence::Comparison,
            Precedence::Comparison => Precedence::Term,
            Precedence::Term => Precedence::Factor,
            Precedence::Factor => Precedence::Unary,
            Precedence::Unary => Precedence::Call,
            Precedence::Call => Precedence::Primary,
            Precedence::Primary => Precedence::Primary,
        }
    }
}

impl<'a> Default for ParseRule<'a> {
    fn default() -> Self {
        Self {
            prefix: Default::default(),
            infix: Default::default(),
            precedence: Precedence::None,
        }
    }
}
