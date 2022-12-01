pub struct Token {
    kind: (),
    line: u32,
    column: u32,
    absolute: usize,
}

impl Token {
    pub const fn new(kind: (), line: u32, column: u32, absolute: usize) -> Token {
        Token {
            kind: (),
            line,
            column,
            absolute,
        }
    }
}
