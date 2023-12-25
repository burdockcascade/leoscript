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

pub struct ParserResult {
    pub tokens: Vec<Token>,
    pub parser_time: std::time::Duration,
}

