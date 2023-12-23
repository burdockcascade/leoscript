pub mod taken;
pub mod matcher;
pub mod sequence;
pub mod mapping;

#[derive(Debug, Clone, PartialEq)]
pub struct ParserInput<'a> {
    pub source: &'a str,
    pub position: ParserPosition,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParserPosition {
    pub(crate) line: usize,
    pub(crate) column: usize,
}

impl ParserInput<'_> {
    pub fn new(source: &str) -> ParserInput {
        ParserInput {
            source,
            position: ParserPosition {
                line: 1,
                column: 1,
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParserError {
    InvalidToken,
    InvalidChar,
    ExpectedToken { expected: char, found: char, position: ParserPosition },
    ExpectedWhitespace { found: char, position: ParserPosition },
    ExpectedAlpha { found: char, position: ParserPosition },
    ExpectedNumeric { found: char, position: ParserPosition },
    ExpectedAlphanumeric { found: char, position: ParserPosition },
    EndOfData { position: ParserPosition },
}