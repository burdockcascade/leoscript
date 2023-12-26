use std::collections::HashMap;

use nom::branch::alt;
use nom::bytes::complete::{tag, tag_no_case, take_until};
use nom::character::complete::{digit1, multispace0};
use nom::combinator::{map, opt};
use nom::IResult;
use nom::multi::separated_list0;
use nom::sequence::{delimited, preceded, separated_pair, tuple};

use crate::compiler::parser::comments::parse_comment;
use crate::compiler::parser::dataobjects::parse_identifier;
use crate::compiler::parser::expressions::parse_expression;
use crate::compiler::parser::functions::{parse_call_function, parse_function, parse_function_code_block};
use crate::compiler::parser::Span;
use crate::compiler::parser::token::{Token, TokenPosition};
use crate::compiler::parser::variables::parse_variable;

const DECIMAL_POINT: &str = ".";

pub fn parse_literal(input: Span) -> IResult<Span, Token> {
    alt((
        parse_array,
        parse_dictionary,
        parse_float,
        parse_integer,
        parse_boolean,
        parse_string,
        parse_null
    ))(input)
}

pub fn parse_null(input: Span) -> IResult<Span, Token> {
    map(
        tag_no_case("null"),
        |_| Token::Null,
    )(input)
}

fn parse_array(input: Span) -> IResult<Span, Token> {
    delimited(
        tag("["),
        map(
            separated_list0(tuple((multispace0, tag(","), multispace0)), parse_literal),
            |items| Token::Array(items),
        ),
        tag("]"),
    )(input)
}

fn parse_dictionary(input: Span) -> IResult<Span, Token> {
    delimited(
        tag("{"),
        delimited(
            multispace0,
            map(
                separated_list0(
                    tuple((multispace0, tag(","), multispace0)),
                    parse_dictionary_item,
                ),
                |items| {
                    let mut dict = HashMap::new();
                    for (key, value) in items {
                        dict.insert(key.to_string(), value);
                    }
                    Token::Dictionary(dict)
                },
            ),
            multispace0,
        ),
        tag("}"),
    )(input)
}

fn parse_dictionary_item(input: Span) -> IResult<Span, (Token, Token)> {
    map(
        tuple((
            alt((parse_string, parse_identifier)),
            preceded(
                delimited(multispace0, tag(":"), multispace0),
                parse_literal,
            )
        )),
        |(key, value)| (key, value),
    )(input)
}

// (-)123
fn parse_integer(input: Span) -> IResult<Span, Token> {
    map(
        tuple((
            opt(tag("-")),
            digit1,
        )),
        |(sign, digits): (Option<Span>, Span)| {
            let mut s = digits.fragment().to_string();
            if sign.is_some() {
                s.insert(0, '-');
            }
            Token::Integer(s.parse::<i64>().unwrap())
        },
    )(input)
}

// (-)123.456
fn parse_float(input: Span) -> IResult<Span, Token> {
    map(
        tuple((
            opt(tag("-")),
            separated_pair(
                digit1,
                tag(DECIMAL_POINT),
                digit1,
            )
        )),
        |(sign, (int, frac)): (Option<Span>, (Span, Span))| {
            let mut s = int.fragment().to_string();
            s.push('.');
            s.push_str(frac.fragment());
            if sign.is_some() {
                s.insert(0, '-');
            }
            Token::Float(s.parse::<f64>().unwrap())
        },
    )(input)
}

fn parse_boolean(input: Span) -> IResult<Span, Token> {
    alt((
        map(tag_no_case("true"), |_| Token::Bool(true)),
        map(tag_no_case("false"), |_| Token::Bool(false))
    ))(input)
}

fn parse_string(input: Span) -> IResult<Span, Token> {
    map(
        delimited(
            tag("\""),
            take_until("\""),
            tag("\""),
        ),
        |s: Span| Token::String(s.fragment().to_string()),
    )(input)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_integer() {
        let (_, token) = parse_integer(Span::new("123")).unwrap();
        assert_eq!(token, Token::Integer(123));
    }

    #[test]
    fn test_parse_negative_integer() {
        let (_, token) = parse_integer(Span::new("-123")).unwrap();
        assert_eq!(token, Token::Integer(-123));
    }

    #[test]
    fn test_parse_float() {
        let (_, token) = parse_float(Span::new("123.456")).unwrap();
        assert_eq!(token, Token::Float(123.456));
    }

    #[test]
    fn test_parse_negative_float() {
        let (_, token) = parse_float(Span::new("-123.456")).unwrap();
        assert_eq!(token, Token::Float(-123.456));
    }

    #[test]
    fn test_parse_boolean_false() {
        let (_, token) = super::parse_boolean(Span::new("true")).unwrap();
        assert_eq!(token, Token::Bool(true));
    }

    #[test]
    fn test_parse_boolean_true() {
        let (_, token) = super::parse_boolean(Span::new("false")).unwrap();
        assert_eq!(token, Token::Bool(false));
    }

    #[test]
    fn test_parse_array() {
        let (_, token) = parse_array(Span::new("[1, true, 3.4]")).unwrap();

        assert_eq!(token, Token::Array(vec![
            Token::Integer(1),
            Token::Bool(true),
            Token::Float(3.4),
        ]));
    }

    #[test]
    fn test_parse_dictionary() {
        let input = r#"{
            "a": 1,
            "b": [true, false, false, true],
            "c": {
                "d": "hello",
                "e": "world",
                "f": {
                    "g": 1,
                    "h": 2,
                    "i": true
                }
            }
        }"#;

        let (_, token) = parse_dictionary(Span::new(input)).unwrap();

        assert_eq!(token, Token::Dictionary(HashMap::from([
            (String::from("a"), Token::Integer(1)),
            (String::from("b"), Token::Array(vec![
                Token::Bool(true),
                Token::Bool(false),
                Token::Bool(false),
                Token::Bool(true),
            ])),
            (String::from("c"), Token::Dictionary(HashMap::from([
                (String::from("d"), Token::String(String::from("hello"))),
                (String::from("e"), Token::String(String::from("world"))),
                (String::from("f"), Token::Dictionary(HashMap::from([
                    (String::from("g"), Token::Integer(1)),
                    (String::from("h"), Token::Integer(2)),
                    (String::from("i"), Token::Bool(true)),
                ])))
            ])))
        ])));
    }

}