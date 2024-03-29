mod color;

use crate::color::*;
use ast::prelude::Span;
use std::cell::Ref;
use std::cell::RefCell;
use std::fmt::{self, Display};
use std::iter::repeat;
use std::rc::Rc;

#[derive(Debug)]
pub struct Diagnostic {
    pub msg: String,
    pub level: Level,
    span: Span,
}

#[derive(Debug, PartialEq)]
pub enum Level {
    Warn,
    Error,
    RunTimeError,
}

impl Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Level::Warn => write!(f, "{}", Yellow.bold().paint("warning")),
            Level::Error => write!(f, "{}", Red.bold().paint("error")),
            Level::RunTimeError => write!(f, "{}", Purple.bold().paint("Runtime Error")),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Reporter {
    diagnostics: Rc<RefCell<Vec<Diagnostic>>>,
}

impl Reporter {
    pub fn new() -> Reporter {
        Self::default()
    }

    pub fn has_error(&self) -> bool {
        !self.diagnostics.borrow().is_empty()
    }

    pub fn remove_error(&mut self) {
        self.diagnostics.borrow_mut().pop();
    }

    pub fn error<T: Into<String>>(&self, msg: T, span: Span) {
        self.diagnostics.borrow_mut().push(Diagnostic {
            msg: msg.into(),
            span,
            level: Level::Error,
        })
    }

    pub fn run_time_error<T: Into<String>>(&self, msg: T, span: Span) {
        self.diagnostics.borrow_mut().push(Diagnostic {
            msg: msg.into(),
            span,
            level: Level::RunTimeError,
        })
    }

    pub fn warn<T: Into<String>>(&self, msg: T, span: Span) {
        self.diagnostics.borrow_mut().push(Diagnostic {
            msg: msg.into(),
            span,
            level: Level::Warn,
        })
    }

    pub fn emit(&self, input: &str) {
        for diagnostic in self.diagnostics.borrow().iter() {
            print(input, diagnostic)
        }
    }

    pub fn diagnostics(&self) -> Ref<Vec<Diagnostic>> {
        self.diagnostics.borrow()
    }
}

impl Default for Reporter {
    fn default() -> Self {
        Self {
            diagnostics: Rc::new(RefCell::new(Vec::new())),
        }
    }
}

fn print(input: &str, d: &Diagnostic) {
    let prefix = Blue.paint("| ");

    println!("{}: {}", d.level, Fixed(252).bold().paint(d.msg.clone()));

    let span = d.span;

    let start_line = if span.start.line >= 4 {
        span.start.line - 4
    } else {
        0
    };

    for (idx, line) in input.lines().enumerate().skip(start_line as usize) {
        let line = line;
        let line_idx = idx + 1;
        println!("{:>4} {}{}", line_idx, prefix, line);
        if line_idx == span.start.line as usize {
            let end = if line_idx == span.end.line as usize {
                span.end.column as usize
            } else {
                line.len()
            };
            let carets = repeat_string("^", end - span.start.column as usize);

            let carets = match d.level {
                Level::Warn => Yellow.bold().paint(carets),
                Level::Error => Red.bold().paint(carets),
                Level::RunTimeError => Purple.bold().paint(carets),
            };

            if span.start.column != 0 {
                let whitespace = repeat_string(" ", span.start.column as usize - 1);
                println!("     {}{}{}", prefix, whitespace, carets);
            }
        } else if line_idx == span.end.line as usize {
            let carets = repeat_string("^", span.end.column as usize);
            let carets = match d.level {
                Level::Warn => Yellow.bold().paint(carets),
                Level::Error => Red.bold().paint(carets),
                Level::RunTimeError => Purple.bold().paint(carets),
            };
            println!("     {}{}", prefix, carets);
        } else if line_idx > span.start.line as usize
            && line_idx < span.end.line as usize
            && !line.is_empty()
        {
            let carets = repeat_string("^", line.len());
            let carets = match d.level {
                Level::Warn => Yellow.bold().paint(carets),
                Level::Error => Red.bold().paint(carets),
                Level::RunTimeError => Purple.bold().paint(carets),
            };
            println!("     {}{}", prefix, carets);
        }

        if line_idx >= span.end.line as usize + 3 {
            break;
        }
    }

    println!()
}

fn repeat_string(s: &str, count: usize) -> String {
    repeat(s).take(count).collect()
}
