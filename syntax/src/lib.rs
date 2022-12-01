pub fn add(left: usize, right: usize) -> usize {
    left + right
}

mod token;

pub type ParserResult<T> = Result<T, ()>;
pub use token::Token;

pub struct Parser<'a> {
    input: &'a str,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
