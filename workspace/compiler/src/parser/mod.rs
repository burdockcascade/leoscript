use nom_locate::LocatedSpan;

use crate::parser::token::{Token, TokenPosition};

mod expressions;
mod literal;
mod dataobjects;
mod variables;
mod comments;
pub(crate) mod token;
mod functions;
mod logic;
mod loops;
pub mod script;

type Span<'a> = LocatedSpan<&'a str>;
const DOT_OPERATOR: &str = ".";

#[derive(Debug, PartialEq)]
pub enum ParserError {
    UnableToParseTokens,
    ExpectedBlockEnd,
    NothingToParse,
    InvalidImportReference { position: TokenPosition, name: String },
    InvalidIdentifier { position: TokenPosition, name: String },
}

pub struct ParserResult {
    pub tokens: Vec<Token>,
    pub errors: Vec<ParserError>,
    pub parser_time: std::time::Duration,
}

