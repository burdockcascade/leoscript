use nom::branch::alt;
use nom::bytes::complete::{tag, tag_no_case};
use nom::character::complete::{multispace0, multispace1};
use nom::combinator::map;
use nom::IResult;
use nom::multi::{many0, separated_list1};
use nom::sequence::{delimited, preceded, terminated};
use crate::parser::{DOT_OPERATOR, ParseError, ParserResult, Span};
use crate::parser::dataobjects::{parse_class, parse_enum, parse_identifier, parse_module};
use crate::parser::functions::parse_function;
use crate::parser::token::{Token, TokenPosition};

pub fn parse_script(input: &str) -> Result<ParserResult, ParseError> {
    let start_parser_timer = std::time::Instant::now();

    let result = many0(
        delimited(
            multispace0,
            alt((
                parse_import,
                parse_function,
                parse_class,
                parse_module,
                parse_enum
            )),
            multispace0,
        )
    )(Span::new(input));

    match result {
        Ok((_, tokens)) => {
            Ok(ParserResult {
                tokens,
                errors: Vec::new(),
                parser_time: start_parser_timer.elapsed(),
            })
        }
        Err(err) => Err(ParseError::UnableToParseTokens),
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
            source
        },
    )(input)
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_parse_import() {
        let (_, token) = parse_import(Span::new("import foo.bar")).unwrap();
        assert_eq!(token, Token::Import {
            position: TokenPosition { line: 1, column: 1 },
            source: vec![
                Token::Identifier {
                    position: TokenPosition { line: 1, column: 8 },
                    name: String::from("foo")
                },
                Token::Identifier {
                    position: TokenPosition { line: 1, column: 12 },
                    name: String::from("bar")
                },
            ],
        })
    }

}