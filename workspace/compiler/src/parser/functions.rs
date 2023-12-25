use nom::branch::alt;
use nom::bytes::complete::{tag, tag_no_case};
use nom::character::complete::{multispace0, multispace1};
use nom::combinator::{map, opt};
use nom::IResult;
use nom::multi::{many0, separated_list0};
use nom::sequence::{delimited, preceded, terminated, tuple};
use nom_locate::position;

use crate::parser::comments::parse_comment;
use crate::parser::dataobjects::{parse_identifier, parse_identifier_chain};
use crate::parser::expressions::parse_expression;
use crate::parser::logic::{parse_if_chain, parse_match_statement};
use crate::parser::loops::{parse_break, parse_continue, parse_for_in_loop, parse_for_to_step, parse_while_loop};
use crate::parser::Span;
use crate::parser::token::{Token, TokenPosition};
use crate::parser::variables::{parse_assignment, parse_variable};

// function name(param1, param2) end
pub fn parse_function(input: Span) -> IResult<Span, Token> {
    map(
        terminated(
            tuple((
                opt(tuple((tag_no_case("static"), multispace1))),
                preceded(tuple((tag_no_case("function"), multispace1)), parse_identifier),
                delimited(multispace0, parse_parameters, multispace1),
                parse_function_code_block
            )),
            tag_no_case("end"),
        ),
        |(is_static, func_name, params, body)| Token::Function {
            position: TokenPosition::new(&input),
            function_name: Box::from(func_name),
            is_static: is_static.is_some(),
            scope: None,
            return_type: None,
            input: params,
            body,
        },
    )(input)
}

fn parse_lambda(input: Span) -> IResult<Span, Token> {
    map(
        delimited(
            tag_no_case("function"),
            tuple((
                terminated(parse_parameters, multispace0),
                parse_function_code_block
            )),
            tag_no_case("end"),
        ),
        |(params, body)| Token::AnonFunction {
            position: TokenPosition::new(&input),
            input: params,
            body,
        },
    )(input)
}

pub fn parse_function_code_block(input: Span) -> IResult<Span, Vec<Token>> {
    many0(
        delimited(
            multispace0,
            alt((
                    parse_comment,
                    parse_variable,
                    parse_assignment,
                    parse_call_function,
                    parse_if_chain,
                    parse_match_statement,
                    parse_while_loop,
                    parse_for_in_loop,
                    parse_for_to_step,
                    parse_identifier_chain,
                    parse_break,
                    parse_continue,
                parse_function_return
            )),
            multispace0)
    )(input)
}

fn parse_parameters(input: Span) -> IResult<Span, Vec<Token>> {
    delimited(
        tag("("),
        separated_list0(
            tuple((multispace0, tag(","), multispace0)),
            map(
                tuple((
                    position,
                    tuple((
                        parse_identifier,
                        opt(preceded(
                            tuple((multispace0, tag_no_case("as"), multispace0)),
                            parse_identifier,
                        )),
                        opt(preceded(
                            tuple((multispace0, tag("="), multispace0)),
                            parse_expression,
                        )),
                    ))
                )),
                |(pos, (name, as_type, value))| Token::Variable {
                    position: TokenPosition::new(&pos),
                    name: name.to_string(),
                    as_type: as_type.map_or(None, |token| Some(token.to_string())),
                    value: value.map_or(None, |token| Some(Box::from(token))),
                },
            ),
        ),
        tag(")"),
    )(input)
}

pub fn parse_function_return(input: Span) -> IResult<Span, Token> {
    preceded(
        tuple((tag_no_case("return"), multispace0)),
        map(
            opt(parse_expression),
            |value| Token::Return {
                position: TokenPosition::new(&input),
                expr: value.map_or(None, |token| Some(Box::from(token))),
            },
        ),
    )(input)
}

pub fn parse_call_function(input: Span) -> IResult<Span, Token> {
    map(
        tuple((
            parse_identifier,
            delimited(
                tag("("),
                separated_list0(tuple((multispace0, tag(","), multispace0)), parse_expression),
                tag(")"),
            )
        )),
        |(name, params)| Token::Call {
            position: TokenPosition::new(&input),
            name: Box::from(name),
            input: params,
        },
    )(input)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_function() {

        let (_, token) = parse_function(Span::new(r#"function sub(a, b)
            return a - b
        end"#)).unwrap();

        // assert_eq!(token, Token::Function {
        //     position: TokenPosition { line: 1, column: 1 },
        //     function_name: Box::from(Token::Identifier {
        //         position: TokenPosition { line: 1, column: 10 },
        //         name: String::from("sub"),
        //     }),
        //     is_static: false,
        //     scope: None,
        //     return_type: None,
        //     input: vec![
        //         Token::Variable {
        //             position: TokenPosition { line: 1, column: 14 },
        //             name: String::from("a"),
        //             as_type: None,
        //             value: None,
        //         },
        //         Token::Variable {
        //             position: TokenPosition { line: 1, column: 17 },
        //             name: String::from("b"),
        //             as_type: None,
        //             value: None,
        //         },
        //     ],
        //     body: vec![
        //         Token::Return {
        //             position: TokenPosition { line: 2, column: 13 },
        //             expr: Some(Box::from(Token::Sub {
        //                 expr1: Box::from(Token::Identifier {
        //                     position: TokenPosition { line: 2, column: 20 },
        //                     name: String::from("a"),
        //                 }),
        //                 expr2: Box::from(Token::Identifier {
        //                     position: TokenPosition { line: 2, column: 24 },
        //                     name: String::from("b"),
        //                 }),
        //             })),
        //         }
        //     ],
        // })
    }

    #[test]
    fn test_parse_function_with_no_return_type() {
        let input = r#"static function sub(a, b)
            return a
        end"#;

        let (_, token) = parse_function(Span::new(input)).unwrap();

        assert_eq!(token, Token::Function {
            position: TokenPosition { line: 1, column: 1 },
            function_name: Box::from(Token::Identifier {
                position: TokenPosition { line: 1, column: 17 },
                name: String::from("sub"),
            }),
            is_static: true,
            scope: None,
            return_type: None,
            input: vec![
                Token::Variable {
                    position: TokenPosition { line: 1, column: 21 },
                    name: String::from("a"),
                    as_type: None,
                    value: None,
                },
                Token::Variable {
                    position: TokenPosition { line: 1, column: 24 },
                    name: String::from("b"),
                    as_type: None,
                    value: None,
                },
            ],
            body: vec![
                Token::Return {
                    position: TokenPosition { line: 2, column: 13 },
                    expr: Some(Box::from(Token::Identifier {
                        position: TokenPosition { line: 2, column: 20 },
                        name: String::from("a"),
                    })),
                }
            ],
        })
    }

    #[test]
    fn test_parse_lambda() {
        let (_, token) = parse_lambda(Span::new(r#"function(a,b)
            return a / b
        end"#)).unwrap();

        assert_eq!(token, Token::AnonFunction {
            position: TokenPosition { line: 1, column: 1 },
            input: vec![
                Token::Variable {
                    position: TokenPosition { line: 1, column: 10 },
                    name: String::from("a"),
                    as_type: None,
                    value: None,
                },
                Token::Variable {
                    position: TokenPosition { line: 1, column: 12 },
                    name: String::from("b"),
                    as_type: None,
                    value: None,
                },
            ],
            body: vec![
                Token::Return {
                    position: TokenPosition { line: 2, column: 13 },
                    expr: Some(Box::from(Token::Div {
                        expr1: Box::from(Token::Identifier {
                            position: TokenPosition { line: 2, column: 20 },
                            name: String::from("a"),
                        }),
                        expr2: Box::from(Token::Identifier {
                            position: TokenPosition { line: 2, column: 24 },
                            name: String::from("b"),
                        }),
                    })),
                }
            ],
        })
    }

    #[test]
    fn test_return_with_value() {
        let (_, tokens) = parse_function_return(Span::new("return 123")).unwrap();
        assert_eq!(tokens, Token::Return {
            position: TokenPosition { line: 1, column: 1 },
            expr: Some(Box::from(Token::Integer(123))),
        });
    }

    #[test]
    fn test_return_with_value_chain() {
        let (_, tokens) = parse_function_return(Span::new("return self.name")).unwrap();
        assert_eq!(tokens, Token::Return {
            position: TokenPosition { line: 1, column: 1 },
            expr: Some(Box::from(Token::DotChain {
                position: TokenPosition { line: 1, column: 8 },
                start: Box::from(Token::Identifier {
                    position: TokenPosition { line: 1, column: 8 },
                    name: String::from("self"),
                }),
                chain: vec![
                    Token::Identifier {
                        position: TokenPosition { line: 1, column: 13 },
                        name: String::from("name"),
                    }
                ],
            })),
        });
    }

    #[test]
    fn test_return_with_expression() {
        let (_, tokens) = parse_function_return(Span::new("return 7 * b")).unwrap();
        assert_eq!(tokens, Token::Return {
            position: TokenPosition { line: 1, column: 1 },
            expr: Some(Box::from(Token::Mul {
                expr1: Box::from(Token::Integer(7)),
                expr2: Box::from(Token::Identifier {
                    position: TokenPosition { line: 1, column: 12 },
                    name: String::from("b"),
                }),
            })),
        });
    }

    #[test]
    fn test_return_with_no_value() {
        let (_, tokens) = parse_function_return(Span::new("return")).unwrap();
        assert_eq!(tokens, Token::Return { position: TokenPosition { line: 1, column: 1 }, expr: None });
    }

    #[test]
    fn test_function_call_with_no_parameters() {
        let (_, token) = parse_call_function(Span::new("add()")).unwrap();

        assert_eq!(token, Token::Call {
            position: TokenPosition { line: 1, column: 1 },
            name: Box::from(Token::Identifier {
                position: TokenPosition { line: 1, column: 1 },
                name: String::from("add"),
            }),
            input: vec![],
        })
    }

    #[test]
    fn test_function_call_with_parameters() {
        let (_, token) = parse_call_function(Span::new(r#"add(test(true), myfunc() == b, 1, 3.6, 4 > 6, "hello world")"#)).unwrap();

        assert_eq!(token, Token::Call {
            position: TokenPosition { line: 1, column: 1 },
            name: Box::from(Token::Identifier {
                position: TokenPosition { line: 1, column: 1 },
                name: String::from("add"),
            }),
            input: vec![
                Token::Call {
                    position: TokenPosition { line: 1, column: 5 },
                    name: Box::from(Token::Identifier {
                        position: TokenPosition { line: 1, column: 5 },
                        name: String::from("test"),
                    }),
                    input: vec![
                        Token::Bool(true)
                    ],
                },
                Token::Eq {
                    expr1: Box::from(Token::Call {
                        position: TokenPosition { line: 1, column: 17 },
                        name: Box::from(Token::Identifier {
                            position: TokenPosition { line: 1, column: 17 },
                            name: String::from("myfunc"),
                        }),
                        input: vec![],
                    }),
                    expr2: Box::from(Token::Identifier {
                        position: TokenPosition { line: 1, column: 29 },
                        name: String::from("b"),
                    }),
                },
                Token::Integer(1),
                Token::Float(3.6),
                Token::Gt {
                    expr1: Box::from(Token::Integer(4)),
                    expr2: Box::from(Token::Integer(6)),
                },
                Token::String(String::from("hello world")),
            ],
        })
    }

}