use ast::prelude::{Program, SymbolDB};
pub use resolve::Resolver;

mod resolve;
mod scope_map;

pub fn construct_ir(src: &str, (ast, symbols): (Program, SymbolDB)) -> Option<()> {
    let resolver = Resolver::new(symbols);

    let errors = resolver.resolve_program(&ast);

    if errors.has_error() {
        errors.emit(src);
        None
    } else {
        Some(())
    }
}
