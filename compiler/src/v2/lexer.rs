use super::parser::Parser;
use crate::matches;

use ast::prelude::{Position, Span, Spanned, Token};

pub struct Lexer<'a> {
    pub(crate) src: &'a str,
    pub(crate) lookahead: Option<Position>,
    pub(crate) start: Position,
    pub(crate) end: Position,
}

impl<'a> Lexer<'a> {
    pub fn new(src: &'a str) -> Self {
        Self {
            src,
            lookahead: Some(Position::new(1, 1, 0)),
            start: Position::new(1, 1, 0),
            end: Position::new(1, 1, 0),
        }
    }
    pub fn next_token(&mut self) -> Spanned<Token> {
        self.skip_whitespace();

        if self.is_at_end() {
            return self.make_token(Token::Eof);
        }

        let c = self.advance();

        match c {
            Some((start, ch)) => match ch {
                "(" => self.make_token(Token::LeftParen),
                ")" => self.make_token(Token::RightParen),
                "{" => self.make_token(Token::LeftBrace),
                "}" => self.make_token(Token::RightBrace),
                "," => self.make_token(Token::Comma),
                "." => self.make_token(Token::Dot),
                "-" => self.make_token(Token::Minus),
                "+" => self.make_token(Token::Plus),
                "/" => self.make_token(Token::Slash),
                "*" => self.make_token(Token::Star),
                "?" => self.make_token(Token::QuestionMark),
                ":" => matches!(self, "=", Token::Equal, Token::Colon),
                ";" => self.make_token(Token::SemiColon),
                "!" => matches!(self, "=", Token::BangEqual, Token::Bang),
                "=" => {
                    if self.matches("=") {
                        self.make_token(Token::EqualEqual)
                    } else {
                        self.make_token(Token::Equal)
                    }
                }
                "<" => matches!(self, "=", Token::LessEqual, Token::Less),
                ">" => matches!(self, "=", Token::GreaterEqual, Token::Greater),
                "$" => self.make_token(Token::Dollar),
                "\"" => self.string(start),
                ch if ch >= "0" && ch <= "9" => self.number(start),
                ch if ch >= "a" && ch <= "z" || ch >= "A" && ch <= "Z" || ch == "_" => {
                    self.identifier(start)
                }
                ch => {
                    println!("err {:?}", ch);
                    self.error_token("Unexpected character.")
                }
            },
            None => self.make_token(Token::Eof),
        }
    }

    pub fn is_at_end(&self) -> bool {
        match self.lookahead {
            Some(lookahead) => lookahead.absolute + 1 > self.src.len(),
            None => true,
        }
    }

    pub fn advance(&mut self) -> Option<(Position, &str)> {
        match self.lookahead {
            Some(pos) => {
                if pos.absolute + 1 > self.src.len() {
                    return None;
                }
                let ch = &self.src[pos.absolute..pos.absolute + 1];
                self.start = pos;
                self.end = self.end.shift(ch);
                self.lookahead = match self.lookahead {
                    Some(lookahead) => {
                        if lookahead.absolute + 1 > self.src.len() {
                            None
                        } else {
                            Some(lookahead.shift(ch))
                        }
                    }
                    None => None,
                };

                Some((pos, ch))
            }

            None => None,
        }
    }

    fn error_token(&self, arg: &'a str) -> Spanned<Token> {
        // @TODO set up error reporting
        eprintln!("{}", arg);
        Spanned::new(Token::Error, Span::new(self.start, self.end))
    }

    fn matches(&mut self, expected: &str) -> bool {
        match self.lookahead {
            Some(pos) => {
                if &self.src[pos.absolute..pos.absolute + 1] != expected {
                    return false;
                };

                self.advance();

                true
            }
            None => false,
        }
    }

    fn make_token(&self, ty: Token) -> Spanned<Token> {
        Spanned::new(ty, Span::new(self.start, self.end))
    }

    fn peek(&self) -> Option<&str> {
        match self.lookahead {
            Some(pos) => self.src.get(pos.absolute..pos.absolute + 1),
            None => None,
        }
    }

    fn peek_next(&self) -> Option<&str> {
        if self.is_at_end() {
            return Some("\n");
        }

        self.src
            .get(self.start.absolute + 1..self.start.absolute + 2)
    }

    fn skip_whitespace(&mut self) {
        loop {
            let c = self.peek();

            match c {
                Some(ch) => match ch {
                    " " | "\r" | "\t" => {
                        self.advance();
                    }
                    "\n" => {
                        self.advance();
                    }
                    "/" => {
                        if self.peek_next() == Some("/") {
                            // A comment goes until the end of the line.
                            while self.peek() != Some("\n") && !self.is_at_end() {
                                self.advance();
                            }
                        } else {
                            return;
                        }
                    }
                    _ => break,
                },
                None => break,
            }
        }
    }

    fn string(&mut self, start: Position) -> Spanned<Token> {
        while self.peek() != Some("\"") && !self.is_at_end() {
            self.advance();
        }

        if self.is_at_end() {
            return self.error_token("Unterminated string.");
        }

        self.advance();

        Spanned::new(Token::String, Span::new(start, self.end))
    }

    fn is_digit(&self, ch: Option<&str>) -> bool {
        match ch {
            Some(c) => c >= "0" && c <= "9",
            None => false,
        }
    }

    fn is_alpha(&self, ch: Option<&str>) -> bool {
        match ch {
            Some(ch) => ch >= "a" && ch <= "z" || ch >= "A" && ch <= "Z" || ch == "_",
            None => false,
        }
    }

    fn number(&mut self, start: Position) -> Spanned<Token> {
        while self.is_digit(self.peek()) {
            self.advance();
        }

        if self.peek() == Some(".") && self.is_digit(self.peek_next()) {
            self.advance();

            while self.is_digit(self.peek()) {
                self.advance();
            }
        }

        Spanned::new(Token::Number, Span::new(start, self.end))
    }

    fn identifier(&mut self, start: Position) -> Spanned<Token> {
        while self.is_alpha(self.peek()) || self.is_digit(self.peek()) {
            self.advance();
        }

        Spanned::new(self.identifier_type(start), Span::new(start, self.end))
    }

    fn identifier_type(&self, start: Position) -> Token {
        // We use absolute here to get the proper index
        match self.src.get(start.absolute..start.absolute + 1) {
            Some("a") => self.check_keyword(start, 2, "nd", Token::And),
            Some("c") => {
                if self.start.absolute - start.absolute > 1 {
                    match self.src.get(start.absolute + 1..start.absolute + 2) {
                        Some("l") => self.check_keyword(start.shift("l"), 3, "ass", Token::Class),
                        Some("o") => self.check_keyword(start.shift("o"), 3, "nst", Token::Const),
                        _ => Token::Identifier,
                    }
                } else {
                    Token::Identifier
                }
            }
            Some("e") => self.check_keyword(start, 3, "lse", Token::Else),
            Some("f") => {
                if self.start.absolute - start.absolute > 1 {
                    match self.src.get(start.absolute + 1..start.absolute + 2) {
                        Some("a") => self.check_keyword(start.shift("a"), 3, "lse", Token::False),
                        Some("o") => self.check_keyword(start.shift("o"), 1, "r", Token::For),
                        Some("n") => Token::Fun,
                        _ => Token::Identifier,
                    }
                } else {
                    Token::Identifier
                }
            }

            Some("i") => self.check_keyword(start, 1, "f", Token::If),
            Some("n") => self.check_keyword(start, 2, "il", Token::Nil),
            Some("o") => self.check_keyword(start, 1, "r", Token::Or),
            Some("p") => self.check_keyword(start, 4, "rint", Token::Print),
            Some("r") => self.check_keyword(start, 5, "eturn", Token::Return),
            Some("s") => self.check_keyword(start, 4, "uper", Token::Super),
            Some("t") => {
                if self.start.absolute - start.absolute > 1 {
                    match self.src.get(start.absolute + 1..start.absolute + 2) {
                        Some("h") => self.check_keyword(start.shift("h"), 2, "is", Token::This),
                        Some("r") => match self.src.get(start.absolute + 2..start.absolute + 3) {
                            Some("u") => {
                                self.check_keyword(start.shift("r").shift("u"), 1, "e", Token::True)
                            }
                            Some("a") => self.check_keyword(
                                start.shift("r").shift("a"),
                                2,
                                "it",
                                Token::Trait,
                            ),

                            _ => Token::Identifier,
                        },
                        Some("y") => self.check_keyword(start.shift("y"), 2, "pe", Token::Type),

                        _ => Token::Identifier,
                    }
                } else {
                    Token::Identifier
                }
            }
            Some("l") => self.check_keyword(start, 2, "et", Token::Var),
            Some("w") => self.check_keyword(start, 4, "hile", Token::While),
            _ => Token::Identifier,
        }
    }

    fn check_keyword(&self, start: Position, length: usize, rest: &str, ty: Token) -> Token {
        // start.absolute is ahead of the current start,
        // that's why we do (self.start.absolute - start.absolute)
        if (self.start.absolute - start.absolute) == length
            && self
                .src
                .get(start.absolute + 1..start.absolute + 1 + length)
                == Some(rest)
        {
            return ty;
        }

        Token::Identifier
    }
}
