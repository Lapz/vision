use crate::{scope_map::StackedMap, visitor::Visitor};
use ast::prelude::{
    Const, Expression, Function, ItemKind, Program, Span, Spanned, Statement, SymbolDB, SymbolId,
    Trait, Type, TypeAlias, DEFAULT_TYPES,
};
use errors::Reporter;
use std::collections::HashSet;

#[cfg(test)]
mod test;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum State {
    Declared,
    Defined,
    Read,
}

/// Information at a local variable declared in a block
#[derive(Copy, Debug, Clone, PartialEq, Eq)]
pub struct LocalData {
    state: State,
    reads: usize,
    span: Span,
}

pub struct Resolver {
    items: HashSet<(SymbolId, ItemKind)>,
    exported_items: HashSet<(SymbolId, ItemKind)>,
    reporter: Reporter,
    symbols: SymbolDB,
    data: StackedMap<(SymbolId, ItemKind), LocalData>,
}

impl Resolver {
    pub fn new(mut symbols: SymbolDB) -> Self {
        let mut default_items = HashSet::new();

        for ty in DEFAULT_TYPES {
            default_items.insert((symbols.intern(ty), ItemKind::Type));
        }

        Self {
            reporter: Reporter::new(),
            items: default_items,
            exported_items: HashSet::new(),
            symbols,
            data: StackedMap::new(),
        }
    }

    pub fn add_item(
        &mut self,
        item: &Spanned<SymbolId>,
        kind: ItemKind,
        exported: bool,
        emit_error: bool,
    ) {
        if self.items.contains(&(*item.value(), kind)) {
            let name = self.symbols.lookup(item.value());

            if emit_error {
                self.reporter.error(
                    format!("The name `{}` is defined multiple times", name),
                    item.span(),
                )
            }
        } else {
            if exported {
                self.exported_items.insert((*item.value(), kind));
            }

            self.items.insert((*item.value(), kind));
        }
    }

    /// The resolver takes the ast, checks that all referenced variables etc are defined and then
    /// it will return a typed syntax tree, the typed syntax tree is the ast tree annotated with all types
    pub fn resolve_program(&mut self, program: &Program) -> Reporter {
        // We begin a scope so we can report the top level unused items;
        self.begin_scope();
        // We support forward declarations so grab the fowared references so we can use them later
        for type_alias in &program.type_alias {
            self.declare_item(type_alias.name, ItemKind::Type, false)
        }

        for const_def in &program.consts {
            self.declare_item(const_def.name, ItemKind::Value, false)
        }

        for function in &program.functions {
            self.declare_item(function.name, ItemKind::Value, false)
        }

        for type_alias in &program.type_alias {
            self.visit_type_alias(type_alias);
            self.define(type_alias.name, ItemKind::Type)
        }

        for const_def in &program.consts {
            self.visit_const(const_def);
            self.define(const_def.name, ItemKind::Value)
        }

        for function in &program.functions {
            self.visit_function(function);
            self.define(function.name, ItemKind::Value)
        }

        self.end_scope();

        self.reporter.clone()
    }

    pub fn declare_item(&mut self, ident: Spanned<SymbolId>, kind: ItemKind, exported: bool) {
        if self.data.get(&(*ident, kind)).is_some() {
            let name = self.symbols.lookup(ident.value());

            let msg = format!("Duplicate item `{}`", name);
            self.reporter.error(msg, ident.span());
        }

        let key = (*ident.value(), kind);

        if exported {
            self.exported_items.insert(key);
        }

        self.items.insert(key);

        self.data.insert(
            key,
            LocalData {
                state: State::Declared,
                reads: 0,
                span: ident.span(),
            },
        )
    }

    pub fn declare(&mut self, ident: Spanned<SymbolId>, kind: ItemKind) {
        let key = (*ident, kind);

        if self.data.get(&key).is_some() {
            let name = self.symbols.lookup(ident.value());

            let msg = format!("The identifier `{}` has already been declared.", name);
            self.reporter.warn(msg, ident.span());
        }
        self.data.insert(
            key,
            LocalData {
                state: State::Declared,
                reads: 0,
                span: ident.span(),
            },
        )
    }

    fn begin_scope(&mut self) {
        self.data.begin_scope();
    }

    fn end_scope(&mut self) {
        for ((name, _), state) in self.data.end_scope_iter() {
            let LocalData { reads, state, span } = state;

            let name = self.symbols.lookup(&name);

            if (reads == 0 || state == State::Declared) && name != "main" {
                let msg = format!("Unused variable `{}`", name);
                self.reporter.warn(msg, span)
            }
        }
    }

    fn define(&mut self, name: Spanned<SymbolId>, kind: ItemKind) {
        self.data.update(
            (*name.value(), kind),
            LocalData {
                state: State::Defined,
                reads: 0,
                span: name.span(),
            },
        )
    }
}

impl<'ast, 'a> Visitor<'ast> for Resolver {
    type Output = ();

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
                self.begin_scope();
                for stmt in stmts {
                    self.visit_stmt(stmt)
                }
                self.end_scope();
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
                self.declare(*identifier, ItemKind::Value);

                if let Some(ty) = ty {
                    self.visit_type(ty)
                }

                if let Some(init) = init {
                    self.visit_expr(init);
                }
                self.define(*identifier, ItemKind::Value)
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
            Expression::Identifier(name) => self.visit_name(name, ItemKind::Value),
            Expression::Binary { lhs, rhs, .. } => {
                self.visit_expr(lhs);
                self.visit_expr(rhs)
            }
            Expression::Grouping(expr) => self.visit_expr(expr),
            Expression::Call { callee, args } => {
                self.visit_expr(callee);
                for arg in args {
                    self.visit_expr(arg);
                }
            }
            Expression::Unary { rhs, .. } => self.visit_expr(rhs),
            Expression::Error => {}
        }
    }

    fn visit_function(&mut self, function: &'ast Spanned<Function>) {
        self.begin_scope();

        for param in &function.params {
            self.visit_function_param(param)
        }

        if let Some(returns) = function.returns.as_ref() {
            self.visit_type(returns);
        }

        self.visit_stmt(&function.body);

        self.end_scope();
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
            Type::Identifier(name) => self.visit_name(name, ItemKind::Type),
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

    fn visit_name(&mut self, ident: &'ast Spanned<SymbolId>, kind: ItemKind) {
        let key = (*ident.value(), kind);

        if let Some(state) = self.data.get_mut(&key) {
            state.state = State::Read;
            state.reads += 1;
            return;
        } //check for ident name in local scope

        if !self.items.contains(&key) {
            let msg = format!(
                "Unknown identifier `{}`",
                self.symbols.lookup(ident.value())
            );

            self.reporter.error(msg, ident.span())
        }
    }

    fn visit_function_param(&mut self, param: &'ast Spanned<ast::prelude::FunctionParam>) {
        self.visit_type(&param.ty)
    }
}
