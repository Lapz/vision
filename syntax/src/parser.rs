use std::collections::HashMap;

use crate::hashmap;

use super::lexer::Lexer;
use ast::prelude::{Expression, Position, Program, Span, Spanned, SymbolDB, Token};
use errors::Reporter;
pub struct Parser<'a> {
    pub(crate) src: &'a str,
    pub(crate) lexer: Lexer<'a>,
    pub(crate) had_error: bool,
    pub(crate) panic_mode: bool,
    pub(crate) current: Spanned<Token>,
    pub(crate) prev: Spanned<Token>,
    pub(crate) reporter: Reporter,
    pub(crate) rules: HashMap<Token, ParseRule<'a>>,
    pub(crate) symbols: SymbolDB,
}

#[derive(Clone, Copy)]
pub struct ParseRule<'a> {
    pub(crate) prefix: Option<fn(&mut Parser<'a>) -> Spanned<Expression>>,
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
            reporter: Reporter::new(),
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
            symbols: SymbolDB::default(),
            rules: hashmap! {
                Token::LeftParen => ParseRule {
                        prefix: Some(Parser::grouping),
                        infix:  Some(Parser::call),
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
                Token::Assignment => ParseRule {
                    prefix: None,
                    infix: Some(Parser::binary),
                    precedence: Precedence::Assignment,
                },
                Token::Equal => ParseRule::default(),
                Token::LeftBrace => ParseRule::default(),
                Token::RightBrace => ParseRule::default(),
                Token::Comma => ParseRule::default(),
                Token::Dot => ParseRule::default(),
                Token::Colon => ParseRule::default(),
                Token::LeftBracket => ParseRule::default(),
                Token::RightBracket => ParseRule::default(),
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

            self.error_at_current("Unexpected token")
        }
    }

    pub fn parse(mut self) -> Option<(Program, SymbolDB)> {
        let mut program = Program::new();

        while !self.match_token(Token::Eof) {
            if self.match_token(Token::Const) {
                program.add_const(self.const_declaration())
            } else if self.match_token(Token::Fun) {
                let fun = self.fn_declaration();
                program.add_fn(fun)
            } else if self.match_token(Token::Type) {
                program.add_type_alias(self.type_alias())
            } else if self.match_token(Token::Trait) {
                self.trait_declaration()
            }

            self.synchronize();
        }

        self.reporter.emit(self.src);

        if self.had_error {
            None
        } else {
            Some((program, self.symbols))
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
