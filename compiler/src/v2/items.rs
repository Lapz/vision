use ast::prelude::{Spanned, Statement};

use super::Parser;

impl<'a> Parser<'a> {
    pub(crate) fn const_declaration(&mut self) {
        todo!()
    }

    pub(crate) fn statement(&mut self) -> Spanned<Statement> {
        self.expression_statement()
    }
    pub(crate) fn fn_declaration(&mut self) {
        todo!()
    }

    pub(crate) fn trait_declaration(&mut self) {
        todo!()
    }
}
