use ast::prelude::{Program, SymbolDB};
pub use resolve::Resolver;

mod ctx;
mod resolve;
mod scope_map;

pub fn construct_ir(src: &str, (ast, symbols): (Program, SymbolDB)) {
    let resolver = Resolver::new(symbols);

    let errors = resolver.resolve_program(&ast);

    errors.emit(src);
}
