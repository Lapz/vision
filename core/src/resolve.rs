use crate::{
    ast::resolved::{self as r},
    scope_map::StackedMap,
    visitor::Visitor,
};
use ::ast::prelude::{self as a, ItemKind, Span, Spanned, SymbolDB, SymbolId, DEFAULT_TYPES};
use errors::Reporter;
use std::collections::HashSet;

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
    pub fn resolve_program(&mut self, program: &a::Program) -> Reporter {
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

impl<'ast> Resolver {
    fn visit_stmt(&mut self, stmt: &'ast Spanned<a::Statement>) {
        match stmt.value() {
            a::Statement::Expression(expr) => self.visit_expr(expr),
            a::Statement::While { cond, body } => {
                self.visit_expr(cond);
                self.visit_stmt(body);
            }
            a::Statement::If { cond, then, else_ } => {
                self.visit_expr(cond);
                self.visit_stmt(then);

                if let Some(else_) = else_ {
                    self.visit_stmt(else_)
                }
            }
            a::Statement::Block(stmts) => {
                self.begin_scope();
                for stmt in stmts {
                    self.visit_stmt(stmt)
                }
                self.end_scope();
            }
            a::Statement::Return(expr) => {
                if let Some(expr) = expr {
                    self.visit_expr(expr);
                }
            }
            a::Statement::Break | a::Statement::Continue => {}
            a::Statement::Let {
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

    fn visit_expr(&mut self, expression: &'ast Spanned<a::Expression>) {
        match expression.value() {
            a::Expression::Literal(_) => {}
            a::Expression::Ternary { cond, lhs, rhs } => {
                self.visit_expr(cond);
                self.visit_expr(lhs);
                self.visit_expr(rhs)
            }
            a::Expression::Identifier(name) => self.visit_name(name, ItemKind::Value),
            a::Expression::Binary { lhs, rhs, .. } => {
                self.visit_expr(lhs);
                self.visit_expr(rhs)
            }
            a::Expression::Grouping(expr) => self.visit_expr(expr),
            a::Expression::Call { callee, args } => {
                self.visit_expr(callee);
                for arg in args {
                    self.visit_expr(arg);
                }
            }
            a::Expression::Unary { rhs, .. } => self.visit_expr(rhs),
            a::Expression::Error => {}
        }
    }

    fn visit_function(&mut self, function: &'ast Spanned<a::Function>) {
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

    fn visit_const(&mut self, const_: &'ast Spanned<a::Const>) {
        if let Some(ref ty) = const_.ty {
            self.visit_type(ty)
        }
        self.visit_expr(&const_.initializer);
    }

    fn visit_trait(&mut self, trait_: &'ast Spanned<a::Trait>) {
        unimplemented!();
    }

    fn visit_type_alias(&mut self, type_: &'ast Spanned<a::TypeAlias>) {
        self.visit_type(&type_.ty);
    }

    fn visit_type(&mut self, type_: &'ast Spanned<a::Type>) -> Spanned<r::Type> {
        let span = type_.span();
        match type_.value() {
            a::Type::Identifier(name) => {
                Spanned::new(r::Type::Named(self.visit_name(name, ItemKind::Type)), span)
            }
            a::Type::Array { ty, .. } => self.visit_type(ty),
            a::Type::Function { params, returns } => {
                let mut resolved_params = Vec::with_capacity(params.len());
                for param in params {
                    resolved_params.push(self.visit_type(param));
                }

                let resolved_return = if let Some(returns) = returns {
                    self.visit_type(returns)
                } else {
                    Spanned::new(r::Type::Void, span)
                };

                Spanned::new(
                    r::Type::Function {
                        params: resolved_params,
                        returns: Box::new(resolved_return),
                    },
                    span,
                )
            }
            a::Type::Error => Spanned::new(r::Type::Error, span),
            a::Type::Void => Spanned::new(r::Type::Void, span),
        }
    }

    fn visit_name(&mut self, ident: &'ast Spanned<SymbolId>, kind: ItemKind) -> Spanned<SymbolId> {
        let key = (*ident.value(), kind);

        let name = Spanned::new(key.0, ident.span());

        if let Some(state) = self.data.get_mut(&key) {
            state.state = State::Read;
            state.reads += 1;
            return name;
        } //check for ident name in local scope

        if !self.items.contains(&key) {
            let msg = format!(
                "Unknown identifier `{}`",
                self.symbols.lookup(ident.value())
            );

            self.reporter.error(msg, ident.span());

            return Spanned::new(self.symbols.intern("?"), ident.span());
        }

        name
    }

    fn visit_function_param(&mut self, param: &'ast Spanned<ast::prelude::FunctionParam>) {
        self.visit_type(&param.ty);
    }
}

#[cfg(test)]
mod test {
    use ast::prelude::ItemKind;
    use errors::Level;
    use syntax::Parser;

    use crate::Resolver;

    struct ExpectedDiagnostic {
        level: Level,
        msg: &'static str,
    }

    macro_rules! setup_reporter {
        ($file:expr) => {{
            let file = $file;

            let parser = Parser::new(file);

            let (program, symbols) = parser.parse().unwrap();

            let mut resolver = Resolver::new(symbols);

            let errors = resolver.resolve_program(&program);

            (errors, resolver)
        }};
    }

    macro_rules! assert_diagnostics {
        ($expected:expr,$reporter:ident) => {{
            let expected = $expected;

            let mut found = 0;

            for diagnostic in $reporter.diagnostics().iter() {
                for exp in expected.iter() {
                    if diagnostic.level == exp.level && diagnostic.msg == exp.msg {
                        found += 1;
                    }
                }
            }

            assert_eq!(found, expected.len())
        }};
    }

    #[test]
    fn it_works() {
        let (reporter, _) = setup_reporter!(
            "fn main() {
                let a := 10;
                let b := 10;


                return a+b;
            }"
        );

        assert!(!reporter.has_error())
    }

    #[test]
    fn it_has_different_environments_for_types() {
        let (_, mut resolver) = setup_reporter!(
            "type a = number;
                fn main() {
                    let a:a := 10;
                    let b:a := 20;
                    return a+b;
                }"
        );

        let a = resolver.symbols.intern("a");

        assert!(resolver.items.get(&(a, ItemKind::Type)).is_some());
        assert!(resolver.items.get(&(a, ItemKind::Type)).is_some())
    }

    #[test]
    fn it_resolves_functions_and_types() {
        let (_, mut resolver) = setup_reporter!(
            "type a = number;
                fn main() {
                    let a:a := 10;
                    let b:a := 20;
                    return a+b;
                }"
        );

        let a = resolver.symbols.intern("a");

        let main = resolver.symbols.intern("main");

        assert!(resolver.items.get(&(a, ItemKind::Type)).is_some());
        assert!(resolver.items.get(&(main, ItemKind::Value)).is_some())
    }

    #[test]
    fn it_errors_on_unknown_identifier() {
        let (reporter, _) = setup_reporter!(
            "
                fn main() {
                    let a := 10;
                    let c := 20;

                    return a+b;
                }"
        );

        let expected = [ExpectedDiagnostic {
            level: Level::Error,
            msg: "Unknown identifier `b`",
        }];

        let mut found = 0;

        for diagnostic in reporter.diagnostics().iter() {
            println!("{:?}", diagnostic.msg);
            for exp in expected.iter() {
                if diagnostic.level == exp.level && diagnostic.msg == exp.msg {
                    found += 1;
                }
            }
        }

        assert_eq!(found, expected.len())
    }

    #[test]
    fn it_warns_on_var_shadowing() {
        let (reporter, _) = setup_reporter!(
            "
                fn main() {
                    let a := 10;
                    {
                        let a := 20;
                    }

                    return a;
                }"
        );

        assert_diagnostics!(
            [ExpectedDiagnostic {
                level: Level::Warn,
                msg: "The identifier `a` has already been declared.",
            }],
            reporter
        )
    }

    #[test]
    fn it_warns_on_unused_variables() {
        let (reporter, _) = setup_reporter!(
            "
                 fn main() {
                    let a := 10;
                    let b := 10;

                    return a;
                }"
        );

        assert_diagnostics!(
            [ExpectedDiagnostic {
                level: Level::Warn,
                msg: "Unused variable `b`",
            },],
            reporter
        )
    }

    #[test]
    fn it_does_not_warn_on_main() {
        let (reporter, _) = setup_reporter!("fn main() {}");

        assert!(!reporter.has_error())
    }
}
