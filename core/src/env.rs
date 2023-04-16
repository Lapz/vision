use ast::prelude::{SymbolDB, DEFAULT_TYPES};

/// A struct that holds the type environment and the symbol environment
/// We have a separate environment for types and values
pub struct Env {
    /// the predefined types
    pub(crate) types_db: SymbolDB,
    /// the predefined items i.e functions
    pub(crate) value_db: SymbolDB,
}

pub enum ItemKind {
    Type,
    Value,
}

impl Env {
    pub fn new() -> Self {
        Self {
            types_db: SymbolDB::new_with(DEFAULT_TYPES),
            value_db: SymbolDB::new(),
        }
    }
}
