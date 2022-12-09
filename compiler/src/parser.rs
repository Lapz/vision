use std::collections::HashMap;

use crate::{
    scanner::Scanner,
    token::{Token, TokenType},
};
use vm::{chunk::Chunk, RawObject, Table, Value};
use vm::{op, StringObject};

pub struct Parser<'a> {
    scanner: Scanner<'a>,
    previous: Token<'a>,
    current: Token<'a>,
    pub had_error: bool,
    panic_mode: bool,
    rules: HashMap<TokenType, ParseRule<'a>>,
    current_chunk: Option<Chunk>,
    objects: RawObject,
    table: Table<'a>,
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
impl Precedence {
    fn higher(&self) -> Precedence {
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
    pub fn new(scanner: Scanner<'a>) -> Parser<'a> {
        Parser {
            scanner,
            previous: Token {
                ty: TokenType::Eof,
                lexme: "\0",
                length: 0,
                line: 0,
            },
            current: Token {
                ty: TokenType::Eof,
                lexme: "\0",
                length: 0,
                line: 0,
            },
            had_error: false,
            panic_mode: false,
            current_chunk: Some(Chunk::new()),
            rules: hashmap! {
                    TokenType::LeftParen => ParseRule {
                        prefix: Some(Parser::grouping),
                        infix: None,
                        precedence: Precedence::None,
                    },
                    TokenType::RightParen => ParseRule::default(),
                    TokenType::LeftBrace => ParseRule::default(),
                    TokenType::RightBrace => ParseRule::default(),
                    TokenType::Comma => ParseRule::default(),
                    TokenType::Dot => ParseRule::default(),
                    TokenType::Minus=> ParseRule {
                        prefix: Some(Parser::unary),
                        infix: Some(Parser::binary),
                        precedence: Precedence::Term,
                    },
                    TokenType::Plus => ParseRule {
                        prefix: None,
                        infix: Some(Parser::binary),
                        precedence: Precedence::Term,
                    },

                    TokenType::SemiColon => ParseRule::default(),

                    TokenType::Slash => ParseRule {
                        prefix: None,
                        infix: Some(Parser::binary),
                        precedence: Precedence::Factor,
                    },

                    TokenType::Star => ParseRule {
                        prefix: None,
                        infix: Some(Parser::binary),
                        precedence: Precedence::Factor,
                    },

                    TokenType::Bang => ParseRule {
                        prefix: Some(Parser::unary),
                        infix: None,
                        precedence: Precedence::None,
                    },

                    TokenType::BangEqual => ParseRule {
                        prefix: None,
                        infix: Some(Parser::binary),
                        precedence: Precedence::Equality,
                    },
                    TokenType::Equal => ParseRule::default(),
                    TokenType::EqualEqual => ParseRule {
                        prefix: None,
                        infix: Some(Parser::binary),
                        precedence: Precedence::Equality,
                    },
                    TokenType::Greater => ParseRule {
                        prefix: None,
                        infix: Some(Parser::binary),
                        precedence: Precedence::Comparison,
                    },
                    TokenType::GreaterEqual => ParseRule {
                        prefix: None,
                        infix: Some(Parser::binary),
                        precedence: Precedence::Comparison,
                    },
                    TokenType::Less => ParseRule {
                        prefix: None,
                        infix: Some(Parser::binary),
                        precedence: Precedence::Comparison,
                    },
                    TokenType::LessEqual => ParseRule {
                        prefix: None,
                        infix: Some(Parser::binary),
                        precedence: Precedence::Comparison,
                    },
                    TokenType::Identifier => ParseRule::default(),
                    TokenType::String => ParseRule {
                        prefix: Some(Parser::string),
                        infix: None,
                        precedence: Precedence::None,
                    },
                    TokenType::Number => ParseRule {
                        prefix: Some(Parser::number),
                        infix: None,
                        precedence: Precedence::None,
                    },
                    TokenType::And => ParseRule::default(),
                    TokenType::Class => ParseRule::default(),
                    TokenType::Else => ParseRule::default(),
                    TokenType::False => ParseRule {
                        prefix: Some(Parser::literal),
                        infix: None,
                        precedence: Precedence::None,
                    },
                    TokenType::For => ParseRule::default(),
                    TokenType::Fun => ParseRule::default(),
                    TokenType::If => ParseRule::default(),
                    TokenType::Nil => ParseRule {
                        prefix: Some(Parser::literal),
                        infix: None,
                        precedence: Precedence::None,
                    },
                    TokenType::Or => ParseRule::default(),
                    TokenType::Print => ParseRule::default(),
                    TokenType::Return => ParseRule::default(),
                    TokenType::Super => ParseRule::default(),
                    TokenType::This => ParseRule::default(),
                    TokenType::True => ParseRule {
                        prefix: Some(Parser::literal),
                        infix: None,
                        precedence: Precedence::None,
                    },
                    TokenType::Var => ParseRule::default(),
                    TokenType::While => ParseRule::default(),
                    TokenType::Error => ParseRule::default(),
                    TokenType::Eof => ParseRule::default(),
            },
            objects: std::ptr::null::<RawObject>() as RawObject,
            table: Table::new(),
        }
    }
    pub fn advance(&mut self) {
        std::mem::swap(&mut self.previous, &mut self.current);

        loop {
            self.current = self.scanner.scan_token();

            if self.current.ty != TokenType::Error {
                break;
            }

            self.error_at_current(self.current.lexme);
        }
    }

    fn error_at_current(&mut self, lexme: &str) {
        self.error_at(self.current, lexme);
    }

    fn error(&mut self, msg: &str) {
        self.error_at(self.previous, msg);
    }

    fn error_at(&mut self, token: Token<'a>, msg: &str) {
        if self.panic_mode {
            return;
        }
        self.panic_mode = true;

        eprint!("[line {}] Error", token.line);

        if token.ty == TokenType::Eof {
            eprint!(" at end");
        } else if token.ty == TokenType::Error {
            // Nothing
        } else {
            eprint!(" at '{}.{}'", token.length, token.lexme);
        }

        eprintln!(": {}", msg);

        self.had_error = true;
    }

    pub fn emit_byte(&mut self, byte: u8) {
        self.current_chunk
            .as_mut()
            .expect("Called emit when chunk not started")
            .write(byte, self.previous.line);
    }

    fn emit_bytes(&mut self, byte1: u8, byte2: u8) {
        self.emit_byte(byte1);
        self.emit_byte(byte2);
    }

    pub fn emit_return(&mut self) {
        self.emit_byte(op::RETURN)
    }

    pub(crate) fn expression(&mut self) {
        self.parse_with_precedence(Precedence::Assignment);
    }

    pub(crate) fn number(&mut self) {
        let value = self.previous.lexme.parse::<f64>().unwrap();
        self.emit_constant(Value::number(value));
    }

    pub fn emit_constant(&mut self, value: Value) {
        let constant = self.make_constant(value);
        self.emit_bytes(op::CONSTANT, constant);
    }

    pub(crate) fn consume(&mut self, ty: TokenType, arg: &str) {
        if self.current.ty == ty {
            self.advance();
            return;
        }

        self.error_at_current(arg);
    }

    pub fn end(mut self) -> (Chunk, Table<'a>, RawObject) {
        self.emit_return();

        let current_chunk = self.current_chunk.take().expect("Chunk not started");

        #[cfg(feature = "debug")]
        {
            if !self.had_error {
                current_chunk.disassemble("code");
            }
        }

        (current_chunk, self.table, self.objects)
    }

    fn make_constant(&mut self, value: Value) -> u8 {
        let constant = self
            .current_chunk
            .as_mut()
            .expect("Called emit when chunk not started")
            .add_constant(value);

        if constant > u8::MAX as usize {
            self.error("Too many constants in one chunk.");
            0
        } else {
            constant as u8
        }
    }

    pub(crate) fn grouping(&mut self) {
        self.expression();

        self.consume(TokenType::RightParen, "Expect ')' after expression.");
    }

    pub(crate) fn unary(&mut self) {
        let ty = self.previous.ty;

        self.parse_with_precedence(Precedence::Unary);

        match ty {
            TokenType::Minus => self.emit_byte(op::NEGATE),
            TokenType::Bang => self.emit_byte(op::NOT),
            _ => unreachable!(),
        }
    }

    pub fn binary(&mut self) {
        let ty = self.previous.ty;

        let rule = self.get_rule(ty);

        self.parse_with_precedence(rule.precedence.higher());

        match ty {
            TokenType::BangEqual => self.emit_bytes(op::EQUAL, op::NOT),
            TokenType::EqualEqual => self.emit_byte(op::EQUAL),
            TokenType::Greater => self.emit_byte(op::GREATER),
            TokenType::GreaterEqual => self.emit_bytes(op::LESS, op::NOT),
            TokenType::Less => self.emit_byte(op::LESS),
            TokenType::LessEqual => self.emit_bytes(op::GREATER, op::NOT),
            TokenType::Plus => self.emit_byte(op::ADD),
            TokenType::Minus => self.emit_byte(op::SUBTRACT),
            TokenType::Star => self.emit_byte(op::MULTIPLY),
            TokenType::Slash => self.emit_byte(op::DIVIDE),
            _ => unreachable!(),
        }
    }

    pub fn literal(&mut self) {
        let ty = self.previous.ty;

        match ty {
            TokenType::False => self.emit_byte(op::FALSE),
            TokenType::Nil => self.emit_byte(op::NIL),
            TokenType::True => self.emit_byte(op::TRUE),
            _ => unreachable!(),
        }
    }

    pub fn string(&mut self) {
        let obj = Value::object(StringObject::new(
            &self.previous.lexme[1..self.previous.lexme.len() - 1],
            &mut self.table,
            self.objects,
        ));

        self.objects = obj.as_obj();

        self.emit_constant(obj);
    }

    pub(crate) fn parse_with_precedence(&mut self, precedence: Precedence) {
        self.advance();

        let prefix_rule = self.get_rule(self.previous.ty).prefix;

        match prefix_rule {
            Some(prefix_rule) => prefix_rule(self),
            None => {
                self.error("Expect expression.");
            }
        }

        while precedence <= self.get_rule(self.current.ty).precedence {
            self.advance();

            let infix_rule = self.get_rule(self.previous.ty).infix;

            match infix_rule {
                Some(infix_rule) => infix_rule(self),
                None => {
                    self.error("Expect expression.");
                }
            }
        }
    }

    fn get_rule(&self, ty: TokenType) -> ParseRule<'a> {
        self.rules[&ty]
    }
}

#[derive(Clone, Copy)]
struct ParseRule<'a> {
    prefix: Option<fn(&mut Parser<'a>)>,
    infix: Option<fn(&mut Parser<'a>)>,
    precedence: Precedence,
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
