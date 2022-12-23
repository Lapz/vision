use crate::token::{Token, TokenType};

pub struct Scanner<'a> {
    src: &'a str,
    /// Start pos of the current lexme in the source code string.
    start: usize,
    /// The current pos lexme
    current: usize,
    /// The current line of the source code
    line: usize,
}

macro_rules! matches {
    ($scanner:ident,$char:literal,$lhs:path,$rhs:path) => {{
        if $scanner.matches($char) {
            $scanner.make_token($lhs)
        } else {
            $scanner.make_token($rhs)
        }
    }};
}

impl<'a> Scanner<'a> {
    pub fn new(src: &'a str) -> Scanner {
        Scanner {
            src,
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn scan_token(&mut self) -> Token<'a> {
        self.skip_whitespace();
        self.start = self.current;

        if self.is_at_end() {
            return self.make_token(TokenType::Eof);
        }

        let c = self.advance();

        match c {
            Some(ch) => match ch {
                "(" => self.make_token(TokenType::LeftParen),
                ")" => self.make_token(TokenType::RightParen),
                "{" => self.make_token(TokenType::LeftBrace),
                "}" => self.make_token(TokenType::RightBrace),
                ";" => self.make_token(TokenType::SemiColon),
                "," => self.make_token(TokenType::Comma),
                "." => self.make_token(TokenType::Dot),
                "-" => self.make_token(TokenType::Minus),
                "+" => self.make_token(TokenType::Plus),
                "/" => self.make_token(TokenType::Slash),
                "*" => self.make_token(TokenType::Star),
                "?" => self.make_token(TokenType::QuestionMark),
                ":" => self.make_token(TokenType::Colon),
                "!" => matches!(self, "=", TokenType::BangEqual, TokenType::Bang),
                "=" => matches!(self, "=", TokenType::EqualEqual, TokenType::Equal),
                "<" => matches!(self, "=", TokenType::LessEqual, TokenType::Less),
                ">" => matches!(self, "=", TokenType::GreaterEqual, TokenType::Greater),
                "\"" => self.string(),
                ch if ch >= "0" && ch <= "9" => self.number(),
                ch if ch >= "a" && ch <= "z" || ch >= "A" && ch <= "Z" || ch == "_" => {
                    self.identifier()
                }

                _ => self.error_token("Unexpected character."),
            },
            None => self.make_token(TokenType::Eof),
        }
    }

    const fn is_at_end(&self) -> bool {
        self.current >= self.src.len()
    }

    fn advance(&mut self) -> Option<&str> {
        self.current += 1;

        self.src.get(self.current - 1..self.current)
    }

    fn error_token(&self, arg: &'a str) -> Token<'a> {
        Token {
            ty: TokenType::Error,
            lexme: arg,
            length: self.current - self.start,
            line: self.line,
        }
    }

    fn matches(&mut self, expected: &str) -> bool {
        if self.is_at_end() {
            return false;
        };

        if &self.src[self.current..self.current + 1] != expected {
            return false;
        };

        self.current += 1;

        true
    }

    fn make_token(&self, ty: TokenType) -> Token<'a> {
        let length = self.current - self.start;
        Token {
            ty: ty,
            lexme: &self.src[self.start..self.current],
            length,
            line: self.line,
        }
    }

    fn peek(&self) -> Option<&str> {
        self.src.get(self.current..self.current + 1)
    }

    fn peek_next(&self) -> Option<&str> {
        if self.is_at_end() {
            return Some("\n");
        }

        self.src.get(self.current + 1..self.current + 2)
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
                        self.line += 1;
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

    fn string(&mut self) -> Token<'a> {
        while self.peek() != Some("\"") && !self.is_at_end() {
            if self.peek() == Some("\n") {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            return self.error_token("Unterminated string.");
        }

        self.advance();

        self.make_token(TokenType::String)
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

    fn number(&mut self) -> Token<'a> {
        while self.is_digit(self.peek()) {
            self.advance();
        }

        if self.peek() == Some(".") && self.is_digit(self.peek_next()) {
            self.advance();

            while self.is_digit(self.peek()) {
                self.advance();
            }
        }

        self.make_token(TokenType::Number)
    }

    fn identifier(&mut self) -> Token<'a> {
        while self.is_alpha(self.peek()) || self.is_digit(self.peek()) {
            self.advance();
        }

        self.make_token(self.identifier_type())
    }

    fn identifier_type(&self) -> TokenType {
        match self.src.get(self.start..self.start + 1) {
            Some("a") => self.check_keyword(1, 2, "nd", TokenType::And),
            Some("c") => self.check_keyword(1, 4, "lass", TokenType::Class),
            Some("e") => self.check_keyword(1, 3, "lse", TokenType::Else),
            Some("f") => {
                if self.current - self.start > 1 {
                    match self.src.get(self.start + 1..self.start + 2) {
                        Some("a") => self.check_keyword(2, 3, "lse", TokenType::False),
                        Some("o") => self.check_keyword(2, 1, "r", TokenType::For),
                        Some("u") => self.check_keyword(2, 1, "n", TokenType::Fun),
                        _ => TokenType::Identifier,
                    }
                } else {
                    TokenType::Identifier
                }
            }
            Some("i") => self.check_keyword(1, 1, "f", TokenType::If),
            Some("n") => self.check_keyword(1, 2, "il", TokenType::Nil),
            Some("o") => self.check_keyword(1, 1, "r", TokenType::Or),
            Some("p") => self.check_keyword(1, 4, "rint", TokenType::Print),
            Some("r") => self.check_keyword(1, 5, "eturn", TokenType::Return),
            Some("s") => self.check_keyword(1, 4, "uper", TokenType::Super),
            Some("t") => {
                if self.current - self.start > 1 {
                    match self.src.get(self.start + 1..self.start + 2) {
                        Some("h") => self.check_keyword(2, 2, "is", TokenType::This),
                        Some("r") => self.check_keyword(2, 2, "ue", TokenType::True),
                        _ => TokenType::Identifier,
                    }
                } else {
                    TokenType::Identifier
                }
            }
            Some("v") => self.check_keyword(1, 2, "ar", TokenType::Var),
            Some("w") => self.check_keyword(1, 4, "hile", TokenType::While),
            _ => TokenType::Identifier,
        }
    }

    fn check_keyword(&self, start: usize, length: usize, rest: &str, ty: TokenType) -> TokenType {
        if self.current - self.start == start + length
            && self
                .src
                .get(self.start + start..(self.start + start + length))
                == Some(rest)
        {
            return ty;
        }

        TokenType::Identifier
    }
}
