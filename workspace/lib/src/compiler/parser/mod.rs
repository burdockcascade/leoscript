use nom_locate::LocatedSpan;
use crate::compiler::parser::token::Token;

mod expressions;
mod literal;
mod dataobjects;
mod variables;
mod comments;
pub mod token;
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

