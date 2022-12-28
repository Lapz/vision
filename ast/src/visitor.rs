use crate::{
    expression::{self, Expression},
    items::{Const, Function, FunctionParam, ParamKind, Trait, TypeAlias},
    prelude::{Spanned, Statement, SymbolId, Type},
};

pub trait Visitor<'ast>: Sized {
    fn visit_stmt(&mut self, stmt: &'ast Spanned<Statement>);
    fn visit_expr(&mut self, expression: &'ast Spanned<Expression>);
    fn visit_function(&mut self, function: &'ast Spanned<Function>);
    fn visit_const(&mut self, const_: &'ast Spanned<Const>);
    fn visit_trait(&mut self, trait_: &'ast Spanned<Trait>);
    fn visit_type(&mut self, type_: &'ast Spanned<Type>);
    fn visit_type_alias(&mut self, type_alias: &'ast Spanned<TypeAlias>);
    fn visit_name(&mut self, name: &'ast Spanned<SymbolId>);
    fn visit_function_param(&mut self, param: &'ast Spanned<FunctionParam>, kind: ParamKind);
}
