use std::collections::HashMap;

use crate::{
    local::Compiler,
    scanner::Scanner,
    token::{Token, TokenType},
};
use vm::{chunk::Chunk, op::GET_GLOBAL, RawObject, Table, Value};
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
    table: Table,
    compiler: Option<Compiler<'a>>,
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
                    TokenType::Identifier => ParseRule {
                        prefix: Some(Parser::variable),
                        infix: None,
                        precedence: Precedence::None,
                    },
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
                    TokenType::And => ParseRule {
                        prefix: None,
                        infix: Some(Parser::and),
                        precedence: Precedence::And,
                    },
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
                    TokenType::Or =>  ParseRule {
                        prefix: None,
                        infix: Some(Parser::or),
                        precedence: Precedence::Or,
                    },
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
            compiler: Some(Compiler::new()),
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

    pub(crate) fn number(&mut self, _can_assign: bool) {
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

    pub fn end(mut self) -> (Chunk, Table, RawObject) {
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

    pub(crate) fn grouping(&mut self, _can_assign: bool) {
        self.expression();

        self.consume(TokenType::RightParen, "Expect ')' after expression.");
    }

    pub(crate) fn unary(&mut self, _can_assign: bool) {
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

    pub fn literal(&mut self, _can_assign: bool) {
        let ty = self.previous.ty;

        match ty {
            TokenType::False => self.emit_byte(op::FALSE),
            TokenType::Nil => self.emit_byte(op::NIL),
            TokenType::True => self.emit_byte(op::TRUE),
            _ => unreachable!(),
        }
    }

    pub fn string(&mut self, _can_assign: bool) {
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

        let can_assign = precedence <= Precedence::Assignment;

        match prefix_rule {
            Some(prefix_rule) => prefix_rule(self, can_assign),
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

    pub(crate) fn match_token(&mut self, ty: TokenType) -> bool {
        if !self.check(ty) {
            return false;
        }
        self.advance();

        true
    }

    pub(crate) fn declaration(&mut self) {
        if self.match_token(TokenType::Var) {
            self.var_declaration();
        } else {
            self.statement();
        }

        if self.panic_mode {
            self.synchronize();
        }
    }

    fn statement(&mut self) {
        if self.match_token(TokenType::Print) {
            self.print_statement();
        } else if self.match_token(TokenType::LeftBrace) {
            self.begin_scope();
            self.block();
            self.end_scope();
        } else if self.match_token(TokenType::If) {
            self.if_statement()
        } else if self.match_token(TokenType::While) {
            self.while_statement();
        } else if self.match_token(TokenType::For) {
            self.for_statement();
        } else {
            self.expression_statement();
        }
    }

    fn check(&self, ty: TokenType) -> bool {
        self.current.ty == ty
    }

    fn print_statement(&mut self) {
        self.expression();
        self.consume(TokenType::SemiColon, "Expect ';' after value.");
        self.emit_byte(op::PRINT)
    }

    fn expression_statement(&mut self) {
        self.expression();
        self.consume(TokenType::SemiColon, "Expected ';' after expression.");
        self.emit_byte(op::POP);
    }

    fn synchronize(&mut self) {
        self.panic_mode = false;

        while self.current.ty != TokenType::Eof {
            if self.previous.ty == TokenType::SemiColon {
                return;
            }

            match self.current.ty {
                TokenType::Class
                | TokenType::Fun
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => return,
                _ => {}
            }

            self.advance();
        }
    }

    fn var_declaration(&mut self) {
        let global = self.parse_variable("Expect variable name.");

        if self.match_token(TokenType::Equal) {
            self.expression()
        } else {
            self.emit_byte(op::NIL)
        }

        self.consume(
            TokenType::SemiColon,
            "Expected ';' after variable declaration",
        );

        self.define_variable(global)
    }

    fn parse_variable(&mut self, error_msg: &str) -> u8 {
        self.consume(TokenType::Identifier, error_msg);

        self.declare_variable();

        if self
            .compiler
            .as_ref()
            .expect("Started compiling a block with no compiler")
            .scope_depth
            > 0
        {
            return 0;
        }
        self.identifier_constant(self.previous.lexme)
    }

    fn mark_initialized(&mut self) {
        let current_depth = self.compiler.as_ref().unwrap().scope_depth;

        let slot = self.compiler.as_ref().unwrap().local_count - 1;

        self.compiler.as_mut().unwrap().locals[slot].depth = current_depth
    }
    fn define_variable(&mut self, global: u8) {
        if self.compiler.as_ref().unwrap().scope_depth > 0 {
            self.mark_initialized();
            return;
        }
        self.emit_bytes(op::DEFINE_GLOBAL, global)
    }

    fn identifier_constant(&mut self, lexme: &str) -> u8 {
        let val = Value::object(StringObject::new(lexme, &mut self.table, self.objects));
        self.make_constant(val)
    }

    fn variable(&mut self, can_assign: bool) {
        self.named_variable(self.previous.lexme, can_assign);
    }

    fn named_variable(&mut self, name: &str, can_assign: bool) {
        let get_op;

        let set_op;

        let arg = {
            let arg = self.resolve_local(name);

            if arg.is_none() {
                get_op = op::GET_GLOBAL;
                set_op = op::SET_GLOBAL;
                self.identifier_constant(name)
            } else {
                get_op = op::GET_LOCAL;
                set_op = op::SET_LOCAL;
                arg.unwrap()
            }
        };

        if can_assign && self.match_token(TokenType::Equal) {
            self.expression();
            self.emit_bytes(set_op, arg);
        } else {
            self.emit_bytes(get_op, arg)
        }
    }

    fn begin_scope(&mut self) {
        self.compiler
            .as_mut()
            .expect("Started compiling a block with no compiler")
            .scope_depth += 1
    }

    fn end_scope(&mut self) {
        self.compiler
            .as_mut()
            .expect("Started compiling a block with no compiler")
            .scope_depth -= 1;

        while self.compiler.as_ref().unwrap().local_count > 0
            && self.compiler.as_ref().unwrap().locals
                [self.compiler.as_ref().unwrap().local_count - 1]
                .depth
                > self.compiler.as_ref().unwrap().scope_depth
        {
            self.emit_byte(op::POP);
            self.compiler.as_mut().unwrap().local_count -= 1;
        }
    }

    fn block(&mut self) {
        while !self.check(TokenType::RightBrace) && !self.check(TokenType::Eof) {
            self.declaration();
        }

        self.consume(TokenType::RightBrace, "Expected '}' after block.")
    }

    fn declare_variable(&mut self) {
        if self.compiler.as_ref().unwrap().scope_depth == 0 {
            return;
        }

        for i in (0..self.compiler.as_ref().unwrap().local_count).rev() {
            let local = self.compiler.as_ref().unwrap().locals[i];

            if local.depth != -1 && local.depth < self.compiler.as_ref().unwrap().scope_depth {
                break;
            }

            if local.name.lexme == self.previous.lexme {
                self.error("Already a variable with this name in scope")
            }
        }

        self.add_local(self.previous);
    }

    fn add_local(&mut self, name: Token<'a>) {
        let compiler = self
            .compiler
            .as_mut()
            .expect("Started compiling a block with no compiler");

        if compiler.local_count == 256 {
            self.error("Too many local variables in function");
            return;
        }
        let slot = compiler.local_count;

        compiler.local_count += 1;

        compiler.locals[slot].name = name;
        compiler.locals[slot].depth = -1;

        // todo!()
    }

    fn resolve_local(&mut self, name: &str) -> Option<u8> {
        for i in (0..self.compiler.as_ref().unwrap().local_count).rev() {
            let local = self.compiler.as_ref().unwrap().locals[i];

            if local.name.lexme == name {
                if local.depth == -1 {
                    self.error("Cant'read local variable in its own initializer")
                }
                return Some(i as u8);
            }
        }
        None
    }

    fn if_statement(&mut self) {
        self.consume(TokenType::LeftParen, "Expect '(' after 'if'.");
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after 'if'.");

        let then_jump = self.emit_jump(op::JUMP_IF_FALSE);

        self.statement();

        let else_jump = self.emit_jump(op::JUMP);

        self.patch_jump(then_jump);

        self.emit_byte(op::POP);

        if self.match_token(TokenType::Else) {
            self.statement();
        }

        self.patch_jump(else_jump)
    }

    fn emit_jump(&mut self, jump_if_false: u8) -> usize {
        self.emit_byte(jump_if_false);
        self.emit_bytes(0xff, 0xff);
        self.current_chunk.as_ref().unwrap().code.len() - 2
    }

    fn patch_jump(&mut self, offset: usize) {
        let jump = (self.current_chunk.as_ref().unwrap().code.len() - offset - 2) as u16;

        if jump >= u16::MAX {
            self.error("Too much code to jump over.")
        }

        self.current_chunk.as_mut().unwrap().code[offset] = ((jump >> 8) & 0xff) as u8;

        self.current_chunk.as_mut().unwrap().code[offset + 1] = (jump & 0xff) as u8;
    }

    fn and(&mut self) {
        let end_jump = self.emit_jump(op::JUMP_IF_FALSE);

        self.emit_byte(op::POP);

        self.parse_with_precedence(Precedence::And);

        self.patch_jump(end_jump)
    }

    fn or(&mut self) {
        let else_jump = self.emit_jump(op::JUMP_IF_FALSE);
        let end_jump = self.emit_jump(op::JUMP);

        self.patch_jump(else_jump);

        self.emit_byte(op::POP);

        self.parse_with_precedence(Precedence::Or);

        self.patch_jump(end_jump)
    }

    fn while_statement(&mut self) {
        let loop_start = self.current_chunk.as_ref().unwrap().code.len();

        self.consume(TokenType::LeftParen, "Expected '(' after 'while'.");

        self.expression();

        self.consume(TokenType::RightParen, "Expected ')' after condition");

        let exit_jump = self.emit_jump(op::JUMP_IF_FALSE);

        self.emit_byte(op::POP);

        self.statement();

        self.emit_loop(loop_start);

        self.patch_jump(exit_jump);

        self.emit_byte(op::POP)
    }

    fn emit_loop(&mut self, loop_start: usize) {
        self.emit_byte(op::LOOP);

        let offset = (self.current_chunk.as_ref().unwrap().code.len() - loop_start + 2) as u16;

        if offset > u16::MAX {
            self.error("Loop body too large.");
        }

        self.emit_byte(((offset >> 8) & 0xff) as u8);
        self.emit_byte((offset & 0xff) as u8);
    }

    fn for_statement(&mut self) {
        self.begin_scope();
        self.consume(TokenType::LeftParen, "Expected '(' after 'for'.");

        if self.match_token(TokenType::SemiColon) {
        } else if self.match_token(TokenType::Var) {
            self.var_declaration();
        } else {
            self.expression_statement();
        }

        let mut loop_start = self.current_chunk.as_ref().unwrap().code.len();

        let mut exit_jump: Option<usize> = None;

        if !self.match_token(TokenType::SemiColon) {
            self.expression();

            self.consume(TokenType::SemiColon, "Expected ';' after loop condition");

            exit_jump = Some(self.emit_jump(op::JUMP_IF_FALSE));

            self.emit_byte(op::POP)
        }

        if !self.match_token(TokenType::RightParen) {
            let body_jump = self.emit_jump(op::JUMP);

            let increment_start = self.current_chunk.as_ref().unwrap().code.len();

            self.expression();

            self.emit_byte(op::POP);

            self.consume(TokenType::RightParen, "Expect ')' after for clauses.");

            self.emit_loop(loop_start);

            loop_start = increment_start;

            self.patch_jump(body_jump);
        }

        self.statement();

        self.emit_loop(loop_start);

        if exit_jump.is_some() {
            self.patch_jump(exit_jump.unwrap());
            self.emit_byte(op::POP)
        }

        self.end_scope();
    }
}

#[derive(Clone, Copy)]
struct ParseRule<'a> {
    prefix: Option<fn(&mut Parser<'a>, bool)>,
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
