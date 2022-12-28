use std::collections::HashSet;

use ast::{
    prelude::{
        Const, Expression, Function, ParamKind, Program, Spanned, Statement, SymbolDB, SymbolId,
        Trait, Type, TypeAlias,
    },
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

    pub fn add_item(&mut self, item: &Spanned<SymbolId>, exported: bool, emit_error: bool) {
        if self.items.contains(item.value()) {
            let name = self.symbols.lookup(item.value());

            if emit_error {
                self.reporter.error(
                    format!("The name `{}` is defined multiple times", name),
                    item.span(),
                )
            }
        } else {
            if exported {
                self.exported_items.insert(*item.value());
            }

            self.items.insert(*item.value());
        }
    }

    pub fn resolve_program(mut self, program: &Program) -> Reporter {
        for type_alias in &program.type_alias {
            self.add_item(&type_alias.name, false, true);
        }

        for const_ in &program.consts {
            self.add_item(&const_.name, false, true);
        }

        for function in &program.functions {
            self.add_item(&function.name, false, true);
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

    fn resolve_local(&mut self, ident: &Spanned<SymbolId>) {
        todo!()
    }
}

impl<'ast, 'a> Visitor<'ast> for Resolver {
    fn visit_stmt(&mut self, stmt: &'ast Spanned<Statement>) {
        match stmt.value() {
            Statement::Expression(expr) => self.visit_expr(expr),
            Statement::While { cond, body } => {
                self.visit_expr(cond);
                self.visit_stmt(body);
            }
            Statement::If { cond, then, else_ } => {
                self.visit_expr(cond);
                self.visit_stmt(then);

                if let Some(else_) = else_ {
                    self.visit_stmt(else_)
                }
            }
            Statement::Block(stmts) => {
                for stmt in stmts {
                    self.visit_stmt(stmt)
                }
            }
            Statement::Return(expr) => {
                if let Some(expr) = expr {
                    self.visit_expr(expr);
                }
            }
            Statement::Break | Statement::Continue => {}
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

    fn visit_expr(&mut self, expression: &'ast Spanned<Expression>) {
        match expression.value() {
            Expression::Literal(_) => {}
            Expression::Ternary { cond, lhs, rhs } => {
                self.visit_expr(cond);
                self.visit_expr(lhs);
                self.visit_expr(rhs)
            }
            Expression::Identifier(ident) => self.resolve_local(ident),
            Expression::Binary { lhs, rhs, .. } => {
                self.visit_expr(lhs);
                self.visit_expr(rhs)
            }
            Expression::Grouping(expr) => self.visit_expr(expr),
            Expression::Call { callee, args } => {
                self.resolve_local(callee);
                for arg in args {
                    self.visit_expr(arg);
                }
            }
            Expression::Unary { rhs, op } => self.visit_expr(rhs),
            Expression::Error => {}
        }
    }

    fn visit_function(&mut self, function: &'ast Spanned<Function>) {
        for param in &function.params {
            self.visit_function_param(param, ParamKind::Function)
        }

        if let Some(returns) = function.returns.as_ref() {
            self.visit_type(returns);
        }

        self.visit_stmt(&function.body);
    }

    fn visit_const(&mut self, const_: &'ast Spanned<Const>) {
        if let Some(ref ty) = const_.ty {
            self.visit_type(ty)
        }
        self.visit_expr(&const_.initializer);
    }

    fn visit_trait(&mut self, trait_: &'ast Spanned<Trait>) {}

    fn visit_type_alias(&mut self, type_: &'ast Spanned<TypeAlias>) {
        self.visit_type(&type_.ty);
    }

    fn visit_type(&mut self, type_: &'ast Spanned<Type>) {
        match type_.value() {
            Type::Identifier(name) => self.visit_name(name),
            Type::Array { ty, .. } => self.visit_type(ty),
            Type::Function { params, returns } => {
                for param in params {
                    self.visit_type(param);
                }

                if let Some(returns) = returns {
                    self.visit_type(returns)
                }
            }
            Type::Error | Type::Void => {}
        }
    }

    fn visit_name(&mut self, name: &'ast Spanned<SymbolId>) {}

    fn visit_function_param(
        &mut self,
        param: &'ast Spanned<ast::prelude::FunctionParam>,
        kind: ParamKind,
    ) {
        self.visit_type(&param.ty)
    }
}
