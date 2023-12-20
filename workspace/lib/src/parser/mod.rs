use nom::branch::alt;
use nom::bytes::complete::{tag, tag_no_case};
use nom::character::complete::{multispace0, multispace1};
use nom::combinator::map;
use nom::IResult;
use nom::multi::{many0, separated_list1};
use nom::sequence::{delimited, preceded, terminated};
use nom_locate::LocatedSpan;

use crate::common::error::{ParseError, ScriptError};
use crate::parser::comments::parse_comment;
use crate::parser::dataobjects::{parse_class, parse_enum, parse_identifier, parse_module};
use crate::parser::functions::parse_function;
use crate::parser::token::{Token, TokenPosition};
use crate::script_parse_error;

pub mod token;

mod comments;
mod expressions;
mod literal;
mod functions;
mod loops;
mod dataobjects;
mod logic;
mod variables;

type Span<'a> = LocatedSpan<&'a str>;

const DOT_OPERATOR: &str = ".";

pub struct ParserResult {
    pub tokens: Vec<Token>,
    pub parser_time: std::time::Duration,
}

pub fn parse_script(input: &str) -> Result<ParserResult, ScriptError> {
    let start_parser_timer = std::time::Instant::now();

    let result = many0(
        delimited(
            multispace0,
            alt((
                parse_import,
                parse_comment,
                parse_function,
                parse_class,
                parse_module,
                parse_enum
            )),
            multispace0,
        )
    )(Span::new(input));

    match result {
        Ok((_, tokens)) => Ok(ParserResult {
            tokens,
            parser_time: start_parser_timer.elapsed(),
        }),
        _ => script_parse_error!(ParseError::UnableToParseTokens)
    }
}

fn parse_import(input: Span) -> IResult<Span, Token> {
    map(
        preceded(
            terminated(tag_no_case("import"), multispace1),
            separated_list1(
                tag(DOT_OPERATOR),
                parse_identifier,
            ),
        ),
        |source| Token::Import {
            position: TokenPosition::new(&input),
            source,
        },
    )(input)
}
