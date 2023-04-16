use ast::prelude::{
    Const, Expression, Function, FunctionParam, ItemKind, Spanned, Statement, SymbolId, Trait,
    Type, TypeAlias,
};

pub trait Visitor<'ast>: Sized {
    type Output;
    fn visit_stmt(&mut self, stmt: &'ast Spanned<Statement>) -> Self::Output;
    fn visit_expr(&mut self, expression: &'ast Spanned<Expression>) -> Self::Output;
    fn visit_function(&mut self, function: &'ast Spanned<Function>) -> Self::Output;
    fn visit_const(&mut self, const_: &'ast Spanned<Const>) -> Self::Output;
    fn visit_trait(&mut self, trait_: &'ast Spanned<Trait>) -> Self::Output;
    fn visit_type(&mut self, type_: &'ast Spanned<Type>) -> Self::Output;
    fn visit_type_alias(&mut self, type_alias: &'ast Spanned<TypeAlias>) -> Self::Output;
    fn visit_name(&mut self, name: &'ast Spanned<SymbolId>, kind: ItemKind) -> Self::Output;
    fn visit_function_param(&mut self, param: &'ast Spanned<FunctionParam>) -> Self::Output;
}
