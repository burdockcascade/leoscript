use nom::branch::alt;
use nom::bytes::complete::{tag, take_till, take_until};
use nom::character::complete::{crlf, multispace0};
use nom::combinator::{map, opt};
use nom::IResult;
use nom::sequence::{delimited, terminated};
use crate::parser::Span;
use crate::parser::token::{Token, TokenPosition};

const COMMENT_SINGLE_LINE_TAG: &str = "--";
const COMMENT_MULTI_LINE_START: &str = "--[[";
const COMMENT_MULTI_LINE_END: &str = "]]";

pub fn parse_comment(input: Span) -> IResult<Span, Token> {
    alt((
        parse_multi_line_comment, // this needs to be first, otherwise it will be parsed as a single line comment
        parse_single_line_comment,
    ))(input)
}

fn parse_single_line_comment(input: Span) -> IResult<Span, Token> {
    map(
        delimited(
            terminated(tag(COMMENT_SINGLE_LINE_TAG), multispace0),
            take_till(|c| c == '\n' || c == '\r'),
            opt(crlf),
        ),
        |comment: Span| Token::Comment {
            position: TokenPosition::new(&input),
            text: comment.trim().to_string(),
        },
    )(input)
}

fn parse_multi_line_comment(input: Span) -> IResult<Span, Token> {
    map(
        delimited(
            tag(COMMENT_MULTI_LINE_START),
            take_until("]]"),
            tag(COMMENT_MULTI_LINE_END),
        ),
        |comment: Span| Token::Comment {
            position: TokenPosition::new(&input),
            text: comment.trim().to_string(),
        },
    )(input)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_comment_with_crlf() {
        let (_, token) = parse_comment(Span::new("-- this is a comment\r\n")).unwrap();
        assert_eq!(token, Token::Comment {
            position: TokenPosition { line: 1, column: 1 },
            text: String::from("this is a comment"),
        })
    }

    #[test]
    fn test_parse_comment_till_eof() {
        let (_, token) = parse_comment(Span::new("-- this is a comment")).unwrap();
        assert_eq!(token, Token::Comment {
            position: TokenPosition { line: 1, column: 1 },
            text: String::from("this is a comment"),
        })
    }

    #[test]
    fn test_parse_multi_line_comment() {
        let (_, token) = parse_comment(Span::new(r#"--[[ this is a multi
            line comment ]]"#)).unwrap();
        assert_eq!(token, Token::Comment {
            position: TokenPosition { line: 1, column: 1 },
            text: String::from("this is a multi\n            line comment"),
        })
    }

}