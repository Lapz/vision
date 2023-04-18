use ::ast::prelude::{Program, SymbolDB};
pub use resolve::Resolver;

mod ast {
    pub mod resolved;
}
mod resolve;
mod scope_map;
mod visitor;

pub fn construct_ir(src: &str, (ast, symbols): (Program, SymbolDB)) -> Option<()> {
    let mut resolver = Resolver::new(symbols);

    let errors = resolver.resolve_program(&ast);

    if errors.has_error() {
        errors.emit(src);
        None
    } else {
        Some(())
    }
}
