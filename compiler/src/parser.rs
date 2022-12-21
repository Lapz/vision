use std::collections::HashMap;

use crate::{
    compiler::{Compiler, FunctionType, UpValue},
    scanner::Scanner,
    token::{Token, TokenType},
};
use vm::StringObject;
use vm::{chunk::Chunk, op::Op, FunctionObject, ObjectPtr, RawObject, Table, Value};

pub struct Parser<'a> {
    scanner: Scanner<'a>,
    previous: Token<'a>,
    current: Token<'a>,
    had_error: bool,
    panic_mode: bool,
    rules: HashMap<TokenType, ParseRule<'a>>,
    objects: RawObject,
    table: Table,
    compilers: Vec<Compiler<'a>>,
    current_compiler: usize,
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
        let objects = std::ptr::null::<RawObject>() as RawObject;
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
            rules: hashmap! {
                    TokenType::LeftParen => ParseRule {
                        prefix: Some(Parser::grouping),
                        infix: Some(Parser::call),
                        precedence: Precedence::Call,
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
            objects,
            table: Table::new(),
            compilers: vec![Compiler::new(FunctionType::Script, objects)],
            current_compiler: 0,
        }
    }

    pub fn had_error(&self) -> bool {
        self.had_error
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
        let line = self.previous.line;
        self.current_chunk_mut().write(byte, line);
    }

    fn emit_bytes(&mut self, byte1: u8, byte2: u8) {
        self.emit_byte(byte1);
        self.emit_byte(byte2);
    }

    pub fn emit_return(&mut self) {
        self.emit_bytes(Op::NIL as u8, Op::RETURN as u8);
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
        self.emit_bytes(Op::CONSTANT as u8, constant);
    }

    pub fn start_compiler(&mut self, function: FunctionType) -> usize {
        let mut compiler = Compiler::new(function, self.objects);

        compiler.function.name = Some(StringObject::new(
            self.previous.lexme,
            &mut self.table,
            self.objects,
        ));

        compiler.enclosing = Some(self.current_compiler);
        self.compilers.push(compiler);
        self.current_compiler = self.compilers.len() - 1;
        self.compilers.len() - 1
    }

    pub fn end_compiler(&mut self) -> ObjectPtr<FunctionObject<'a>> {
        self.emit_return();

        let function = self.current_compiler().function.clone();

        #[cfg(feature = "debug")]
        {
            function.chunk.disassemble(match function.name {
                Some(name) => name.chars,
                None => "<script>",
            });
        }

        self.current_compiler = self.current_compiler().enclosing.unwrap();

        function
    }

    #[inline]
    pub fn current_compiler(&self) -> &Compiler<'a> {
        &self.compilers[self.current_compiler]
    }
    #[inline]
    pub fn current_compiler_mut(&mut self) -> &mut Compiler<'a> {
        &mut self.compilers[self.current_compiler]
    }

    pub(crate) fn consume(&mut self, ty: TokenType, arg: &str) {
        if self.current.ty == ty {
            self.advance();
            return;
        }

        self.error_at_current(arg);
    }

    pub fn end(mut self) -> (ObjectPtr<FunctionObject<'a>>, Table, RawObject) {
        self.emit_return();

        #[cfg(feature = "debug")]
        {
            if !self.had_error {
                self.current_chunk()
                    .disassemble(match self.current_compiler().function.name {
                        Some(name) => name.chars,
                        None => "<script>",
                    });
            }
        }

        let function = self.current_compiler().function.clone();

        (function, self.table, self.objects)
    }

    #[inline]
    pub fn current_chunk(&self) -> &Chunk {
        &self.current_compiler().function.chunk
    }

    #[inline]
    pub fn current_chunk_mut(&mut self) -> &mut Chunk {
        &mut self.current_compiler_mut().function.chunk
    }

    fn make_constant(&mut self, value: Value) -> u8 {
        let constant = self.current_chunk_mut().add_constant(value);

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
            TokenType::Minus => self.emit_byte(Op::NEGATE as u8),
            TokenType::Bang => self.emit_byte(Op::NOT as u8),
            _ => unreachable!(),
        }
    }

    pub fn binary(&mut self) {
        let ty = self.previous.ty;

        let rule = self.get_rule(ty);

        self.parse_with_precedence(rule.precedence.higher());

        match ty {
            TokenType::BangEqual => self.emit_bytes(Op::EQUAL as u8, Op::NOT as u8),
            TokenType::EqualEqual => self.emit_byte(Op::EQUAL as u8),
            TokenType::Greater => self.emit_byte(Op::GREATER as u8),
            TokenType::GreaterEqual => self.emit_bytes(Op::LESS as u8, Op::NOT as u8),
            TokenType::Less => self.emit_byte(Op::LESS as u8),
            TokenType::LessEqual => self.emit_bytes(Op::GREATER as u8, Op::NOT as u8),
            TokenType::Plus => self.emit_byte(Op::ADD as u8),
            TokenType::Minus => self.emit_byte(Op::SUBTRACT as u8),
            TokenType::Star => self.emit_byte(Op::MULTIPLY as u8),
            TokenType::Slash => self.emit_byte(Op::DIVIDE as u8),
            _ => unreachable!(),
        }
    }

    pub fn literal(&mut self, _can_assign: bool) {
        let ty = self.previous.ty;

        match ty {
            TokenType::False => self.emit_byte(Op::FALSE as u8),
            TokenType::Nil => self.emit_byte(Op::NIL as u8),
            TokenType::True => self.emit_byte(Op::TRUE as u8),
            _ => unreachable!(),
        }
    }

    pub fn string(&mut self, _can_assign: bool) {
        let obj = Value::object(
            StringObject::new(
                &self.previous.lexme[1..self.previous.lexme.len() - 1],
                &mut self.table,
                self.objects,
            )
            .into(),
        );

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
        } else if self.match_token(TokenType::Fun) {
            self.fun_declaration();
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
        } else if self.match_token(TokenType::Return) {
            self.return_statement();
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
        self.emit_byte(Op::PRINT as u8)
    }

    fn expression_statement(&mut self) {
        self.expression();
        self.consume(TokenType::SemiColon, "Expected ';' after expression.");
        self.emit_byte(Op::POP as u8);
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
            self.emit_byte(Op::NIL as u8)
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

        if self.current_compiler().scope_depth > 0 {
            return 0;
        }
        self.identifier_constant(self.previous.lexme)
    }

    fn mark_initialized(&mut self) {
        let current_depth = self.current_compiler().scope_depth;

        if current_depth == 0 {
            return;
        }

        let slot = self.current_compiler().local_count - 1;

        self.current_compiler_mut().locals[slot].depth = current_depth
    }
    fn define_variable(&mut self, global: u8) {
        if self.current_compiler().scope_depth > 0 {
            self.mark_initialized();
            return;
        }
        self.emit_bytes(Op::DEFINE_GLOBAL as u8, global)
    }

    fn identifier_constant(&mut self, lexme: &str) -> u8 {
        let val = Value::object(StringObject::new(lexme, &mut self.table, self.objects).into());
        self.make_constant(val)
    }

    fn variable(&mut self, can_assign: bool) {
        self.named_variable(self.previous.lexme, can_assign);
    }

    fn named_variable(&mut self, name: &str, can_assign: bool) {
        let get_op;

        let set_op;

        let arg = self.resolve_local(self.current_compiler, name);

        let arg = {
            match arg {
                Some(arg) => {
                    get_op = Op::GET_LOCAL as u8;
                    set_op = Op::SET_LOCAL as u8;
                    arg
                }
                None => match self.resolve_upvalue(self.current_compiler, name) {
                    Some(arg) => {
                        get_op = Op::GET_UPVALUE as u8;
                        set_op = Op::SET_UPVALUE as u8;
                        arg
                    }
                    None => {
                        get_op = Op::GET_GLOBAL as u8;
                        set_op = Op::SET_GLOBAL as u8;
                        self.identifier_constant(name)
                    }
                },
            }
        };

        if can_assign && self.match_token(TokenType::Equal) {
            self.expression();
            self.emit_bytes(set_op, arg);
        } else {
            self.emit_bytes(get_op, arg)
        }
    }

    fn resolve_upvalue(&mut self, compiler_index: usize, name: &str) -> Option<u8> {
        if self.compilers[compiler_index].enclosing.is_none() {
            return None;
        }

        let enclosing = self.compilers[compiler_index].enclosing.unwrap();

        let local = self.resolve_local(enclosing, name);

        if local.is_some() {
            self.compilers[enclosing].locals[local.unwrap() as usize].is_captured = true;
            return Some(self.add_upvalue(compiler_index, local.unwrap(), true));
        }

        let upvalue = self.resolve_upvalue(enclosing, name);

        if upvalue.is_some() {
            return Some(self.add_upvalue(compiler_index, upvalue.unwrap(), false));
        }

        None
    }

    fn add_upvalue(&mut self, compiler_index: usize, local_index: u8, is_local: bool) -> u8 {
        let upvalue_count = self.compilers[compiler_index].function.upvalue_count;

        for i in 0..upvalue_count {
            match self.compilers[compiler_index].upvalues[i] {
                Some(upvalue) => {
                    if upvalue.index == local_index && upvalue.is_local == is_local {
                        return i as u8;
                    }
                }
                None => {}
            }
        }

        if upvalue_count as u8 == u8::MAX {
            self.error("Too many closure variables in function.");
            return 0;
        }

        self.compilers[compiler_index].upvalues[upvalue_count] = Some(UpValue {
            index: local_index,
            is_local,
        });

        self.compilers[compiler_index].function.upvalue_count += 1;

        upvalue_count as u8
    }

    fn begin_scope(&mut self) {
        self.current_compiler_mut().scope_depth += 1
    }

    fn end_scope(&mut self) {
        self.current_compiler_mut().scope_depth -= 1;

        while self.current_compiler().local_count > 0
            && self.current_compiler().locals[self.current_compiler().local_count - 1].depth
                > self.current_compiler().scope_depth
        {
            if self.current_compiler().locals[self.current_compiler().local_count - 1].is_captured {
                self.emit_byte(Op::CLOSE_UPVALUE as u8);
            } else {
                self.emit_byte(Op::POP as u8);
            }

            self.current_compiler_mut().local_count -= 1;
        }
    }

    fn block(&mut self) {
        while !self.check(TokenType::RightBrace) && !self.check(TokenType::Eof) {
            self.declaration();
        }

        self.consume(TokenType::RightBrace, "Expected '}' after block.")
    }

    fn declare_variable(&mut self) {
        if self.current_compiler().scope_depth == 0 {
            return;
        }

        for i in (0..self.current_compiler().local_count).rev() {
            let local = self.current_compiler().locals[i];

            if local.depth != -1 && local.depth < self.current_compiler().scope_depth {
                break;
            }

            if local.name.lexme == self.previous.lexme {
                self.error("Already a variable with this name in scope")
            }
        }

        self.add_local(self.previous);
    }

    fn add_local(&mut self, name: Token<'a>) {
        let compiler = self.current_compiler_mut();

        if compiler.local_count == 256 {
            self.error("Too many local variables in function");
            return;
        }
        let slot = compiler.local_count;

        compiler.local_count += 1;

        compiler.locals[slot].name = name;
        compiler.locals[slot].depth = -1;
    }

    fn resolve_local(&mut self, compiler_index: usize, name: &str) -> Option<u8> {
        for i in (0..self.compilers[compiler_index].local_count).rev() {
            let local = self.compilers[compiler_index].locals[i];

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

        let then_jump = self.emit_jump(Op::JUMP_IF_FALSE as u8);

        self.statement();

        let else_jump = self.emit_jump(Op::JUMP as u8);

        self.patch_jump(then_jump);

        self.emit_byte(Op::POP as u8);

        if self.match_token(TokenType::Else) {
            self.statement();
        }

        self.patch_jump(else_jump)
    }

    fn emit_jump(&mut self, jump_if_false: u8) -> usize {
        self.emit_byte(jump_if_false);
        self.emit_bytes(0xff, 0xff);
        self.current_chunk().code.len() - 2
    }

    fn patch_jump(&mut self, offset: usize) {
        let jump = (self.current_chunk().code.len() - offset - 2) as u16;

        if jump >= u16::MAX {
            self.error("Too much code to jump over.")
        }

        self.current_chunk_mut().code[offset] = ((jump >> 8) & 0xff) as u8;
        self.current_chunk_mut().code[offset + 1] = (jump & 0xff) as u8;
    }

    fn and(&mut self) {
        let end_jump = self.emit_jump(Op::JUMP_IF_FALSE as u8);

        self.emit_byte(Op::POP as u8);

        self.parse_with_precedence(Precedence::And);

        self.patch_jump(end_jump)
    }

    fn or(&mut self) {
        let else_jump = self.emit_jump(Op::JUMP_IF_FALSE as u8);
        let end_jump = self.emit_jump(Op::JUMP as u8);

        self.patch_jump(else_jump);

        self.emit_byte(Op::POP as u8);

        self.parse_with_precedence(Precedence::Or);

        self.patch_jump(end_jump)
    }

    fn while_statement(&mut self) {
        let loop_start = self.current_chunk().code.len();

        self.consume(TokenType::LeftParen, "Expected '(' after 'while'.");

        self.expression();

        self.consume(TokenType::RightParen, "Expected ')' after condition");

        let exit_jump = self.emit_jump(Op::JUMP_IF_FALSE as u8);

        self.emit_byte(Op::POP as u8);

        self.statement();

        self.emit_loop(loop_start);

        self.patch_jump(exit_jump);

        self.emit_byte(Op::POP as u8)
    }

    fn emit_loop(&mut self, loop_start: usize) {
        self.emit_byte(Op::LOOP as u8);

        let offset = (self.current_chunk().code.len() - loop_start + 2) as u16;

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

        let mut loop_start = self.current_chunk().code.len();

        let mut exit_jump: Option<usize> = None;

        if !self.match_token(TokenType::SemiColon) {
            self.expression();

            self.consume(TokenType::SemiColon, "Expected ';' after loop condition");

            exit_jump = Some(self.emit_jump(Op::JUMP_IF_FALSE as u8));

            self.emit_byte(Op::POP as u8)
        }

        if !self.match_token(TokenType::RightParen) {
            let body_jump = self.emit_jump(Op::JUMP as u8);

            let increment_start = self.current_chunk().code.len();

            self.expression();

            self.emit_byte(Op::POP as u8);

            self.consume(TokenType::RightParen, "Expect ')' after for clauses.");

            self.emit_loop(loop_start);

            loop_start = increment_start;

            self.patch_jump(body_jump);
        }

        self.statement();

        self.emit_loop(loop_start);

        if exit_jump.is_some() {
            self.patch_jump(exit_jump.unwrap());
            self.emit_byte(Op::POP as u8)
        }

        self.end_scope();
    }

    fn fun_declaration(&mut self) {
        let global = self.parse_variable("Expect function name.");

        self.mark_initialized();

        self.function(FunctionType::Function);

        self.define_variable(global);
    }

    fn function(&mut self, function: FunctionType) {
        let compiler = self.start_compiler(function);

        self.begin_scope();

        self.consume(TokenType::LeftParen, "Expected '(' after function name.");

        if !self.check(TokenType::RightParen) {
            loop {
                {
                    let function = &mut self.current_compiler_mut().function;

                    function.arity += 1;

                    if function.arity > 255 {
                        self.error_at_current("Can't have more than 255 parameters.")
                    }
                }

                let param = self.parse_variable("Expect parameter name.");

                self.define_variable(param);

                if !self.match_token(TokenType::Comma) {
                    break;
                }
            }
        }

        self.consume(TokenType::RightParen, "Expected ')' after parameters.");

        self.consume(TokenType::LeftBrace, "Expected '{' after function body.");

        self.block();

        let function = self.end_compiler();
        let upvalue_count = function.upvalue_count;

        let constant = self.make_constant(Value::object(function.into()));

        self.emit_bytes(Op::CLOSURE as u8, constant);

        for i in 0..upvalue_count {
            self.emit_byte(if self.compilers[compiler].upvalues[i].unwrap().is_local {
                1
            } else {
                0
            });
            self.emit_byte(self.compilers[compiler].upvalues[i].unwrap().index)
        }
    }

    fn call(&mut self) {
        let arg_count = self.arg_list();
        self.emit_bytes(Op::CALL as u8, arg_count)
    }

    fn arg_list(&mut self) -> u8 {
        let mut count = 0;

        if !self.check(TokenType::RightParen) {
            loop {
                self.expression();

                if count == 255 {
                    self.error("Can't have more than 255 arguments.");
                }

                count += 1;

                if !self.match_token(TokenType::Comma) {
                    break;
                }
            }
        }

        self.consume(TokenType::RightParen, "Expected ')' after arguments");

        count
    }

    fn return_statement(&mut self) {
        if self.current_compiler().compiler_type == FunctionType::Script {
            self.error("Can't return from top-level code.")
        }
        if self.match_token(TokenType::SemiColon) {
            self.emit_return();
        } else {
            self.expression();
            self.consume(TokenType::SemiColon, "Expect ';' after return value.");
            self.emit_byte(Op::RETURN as u8)
        }
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
