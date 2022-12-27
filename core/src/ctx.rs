use std::collections::HashMap;

use ast::prelude::{SymbolId, Type};

use crate::scope_map::StackedMap;

pub struct Ctx {
    pub types: HashMap<SymbolId, Type>,
    pub locals: StackedMap<SymbolId, Type>,
}

impl Ctx {
    pub fn new() -> Self {
        Ctx {
            types: HashMap::new(),
            locals: StackedMap::new(),
        }
    }
}
