use std::collections::HashSet;

use ast::{
    prelude::{Program, Spanned, Statement, SymbolDB, SymbolId},
    visitor::Visitor,
};
use errors::Reporter;

use crate::ctx::Ctx;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum State {
    Declared,
    Defined,
    Read,
}

pub struct Resolver {
    ctx: Ctx,
    items: HashSet<SymbolId>,
    exported_items: HashSet<SymbolId>,
    reporter: Reporter,
    symbols: SymbolDB,
}

impl Resolver {
    pub fn new(symbols: SymbolDB) -> Self {
        Self {
            ctx: Ctx::new(),
            reporter: Reporter::new(),
            items: HashSet::new(),
            exported_items: HashSet::new(),
            symbols,
        }
    }

    pub fn begin_scope(&mut self) {
        self.ctx.locals.begin_scope();
    }
    pub fn end_scope(&mut self) {
        self.ctx.locals.end_scope();
    }

    pub fn add_item(&mut self, item: &Spanned<SymbolId>, exported: bool) {
        if self.items.contains(item.value()) {
            let name = self.symbols.lookup(item.value());

            self.reporter.error(
                format!("The name `{:?}` is defined multiple times", name.unwrap()),
                item.span(),
            )
        } else {
            if exported {
                self.exported_items.insert(*item.value());
            }

            self.items.insert(*item.value());
        }
    }

    pub fn resolve_program(mut self, program: &Program) -> Reporter {
        for type_alias in &program.type_alias {
            self.add_item(&type_alias.name, false);
        }

        for const_ in &program.consts {
            self.add_item(&const_.name, false);
        }

        for function in &program.functions {
            self.add_item(&function.name, false);
        }

        for type_alias in &program.type_alias {
            self.visit_type_alias(type_alias);
        }

        for const_ in &program.consts {
            self.visit_const(const_);
        }

        for function in &program.functions {
            self.visit_function(function)
        }

        self.reporter
    }
}

impl<'ast, 'a> Visitor<'ast> for Resolver {
    fn visit_stmt(&mut self, stmt: &'ast Spanned<ast::prelude::Statement>) {
        match stmt.value() {
            Statement::Expression(_) => {}
            Statement::While { cond, body } => {}
            Statement::If { cond, then, else_ } => {}
            Statement::Block(_) => {}
            Statement::Return(_) => {}
            Statement::Break => {}
            Statement::Continue => {}
            Statement::Let {
                identifier,
                ty,
                init,
            } => {
                if let Some(ty) = ty {
                    self.visit_type(ty)
                }

                if let Some(init) = init {
                    self.visit_expr(init);
                }
            }
        }
    }

    fn visit_expr(&mut self, expression: &'ast Spanned<ast::prelude::Expression>) {
        //{}
    }

    fn visit_function(&mut self, function: &'ast Spanned<ast::prelude::Function>) {
        if let Some(returns) = function.returns.as_ref() {
            self.visit_type(returns);
        }

        self.visit_stmt(&function.body);
    }

    fn visit_const(&mut self, const_: &'ast Spanned<ast::prelude::Const>) {
        //{}
    }

    fn visit_trait(&mut self, trait_: &'ast Spanned<ast::prelude::Trait>) {
        //{}
    }

    fn visit_type_alias(&mut self, type_: &'ast Spanned<ast::prelude::TypeAlias>) {}

    fn visit_type(&mut self, type_: &'ast Spanned<ast::prelude::Type>) {}

    fn visit_name(&mut self, name: &'ast Spanned<SymbolId>) {}
}
