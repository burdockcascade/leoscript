use nom::branch::alt;
use nom::bytes::complete::{tag, tag_no_case};
use nom::character::complete::multispace0;
use nom::combinator::{map, opt};
use nom::IResult;
use nom::sequence::{delimited, preceded, terminated, tuple};

use crate::parser::dataobjects::{parse_identifier, parse_identifier_chain};
use crate::parser::expressions::parse_expression;
use crate::parser::Span;
use crate::parser::token::{Token, TokenPosition};

pub fn parse_variable(input: Span) -> IResult<Span, Token> {
    map(
        preceded(
            tuple((tag_no_case("var"), multispace0)),
            tuple((
                parse_identifier,
                opt(delimited(
                    multispace0,
                    map(
                        preceded(terminated(tag_no_case("as"), multispace0), parse_identifier),
                        |ident| ident.to_string(),
                    ),
                    multispace0,
                )),
                map(
                    opt(preceded(delimited(multispace0, tag("="), multispace0), parse_expression)),
                    |v| {
                        match v {
                            Some(token) => Some(Box::from(token)),
                            None => None
                        }
                    },
                )
            )),
        ),
        |(name, as_type, value)| Token::Variable {
            position: TokenPosition::new(&input),
            name: name.to_string(),
            as_type,
            value,
        },
    )(input)
}

pub fn parse_assignment(input: Span) -> IResult<Span, Token> {
    map(
        tuple((
            alt((parse_identifier_chain, parse_identifier)),
            preceded(delimited(multispace0, tag("="), multispace0), parse_expression)
        )),
        |(name, value)| Token::Assign {
            position: TokenPosition::new(&input),
            ident: Box::from(name),
            value: Box::from(value),
        },
    )(input)
}

#[cfg(test)]
mod test {
    use super::*;

    fn parse_var_with_no_value() {
        let (_, token) = parse_variable(Span::new("var a")).unwrap();

        assert_eq!(token, Token::Variable {
            position: TokenPosition { line: 1, column: 1 },
            name: String::from("a"),
            as_type: None,
            value: None,
        }
        );
    }

    #[test]
    fn parse_var_with_value() {
        let (_, token) = parse_variable(Span::new("var a = 123")).unwrap();

        assert_eq!(token, Token::Variable {
            position: TokenPosition { line: 1, column: 1 },
            name: String::from("a"),
            as_type: None,
            value: Some(Box::from(Token::Integer(123))),
        }
        );
    }

    #[test]
    fn parse_var_with_value_as_integer() {
        let (_, token) = parse_variable(Span::new("var a as Integer = 123")).unwrap();

        assert_eq!(token, Token::Variable {
            position: TokenPosition { line: 1, column: 1 },
            name: String::from("a"),
            as_type: Some(String::from("Integer")),
            value: Some(Box::from(Token::Integer(123))),
        }
        );
    }

}