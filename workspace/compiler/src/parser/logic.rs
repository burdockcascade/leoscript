use nom::branch::alt;
use nom::bytes::complete::tag_no_case;
use nom::character::complete::{multispace0, multispace1};
use nom::combinator::{map, opt};
use nom::IResult;
use nom::multi::many0;
use nom::sequence::{delimited, preceded, separated_pair, terminated, tuple};
use nom_locate::position;

use crate::parser::expressions::parse_expression;
use crate::parser::functions::parse_function_code_block;
use crate::parser::Span;
use crate::parser::token::{Token, TokenPosition};

pub fn parse_if_chain(input: Span) -> IResult<Span, Token> {
    terminated(
        map(
            tuple((
                map(
                    tuple((
                        position,
                        delimited(
                            terminated(tag_no_case("if"), multispace1),
                            parse_expression,
                            tag_no_case("then"),
                        ),
                        parse_function_code_block
                    )),
                    |(pos, cond, block)| Token::If {
                        position: TokenPosition::new(&pos),
                        condition: Box::from(cond),
                        body: block,
                    },
                ),
                many0(map(
                    tuple((
                        position,
                        delimited(
                            terminated(tag_no_case("else if"), multispace1),
                            parse_expression,
                            tag_no_case("then"),
                        ),
                        parse_function_code_block
                    )),
                    |(pos, cond, block)| Token::If {
                        position: TokenPosition::new(&pos),
                        condition: Box::from(cond),
                        body: block,
                    },
                )),
                opt(map(
                    separated_pair(
                        position,
                        tuple((tag_no_case("else"), multispace1)),
                        parse_function_code_block
                    ),
                    |(pos, block)| Token::Else {
                        position: TokenPosition::new(&pos),
                        body: block,
                    },
                ))
            )),
            |(ifstmt, elseifs, elsestmt)| {
                let mut tokens = vec![ifstmt];

                tokens.extend(elseifs);

                if elsestmt.is_some() {
                    tokens.push(elsestmt.unwrap())
                }

                Token::IfChain {
                    position: TokenPosition::new(&input),
                    chain: tokens,
                }
            },
        ),
        tag_no_case("end"),
    )(input)
}

pub fn parse_match_statement(input: Span) -> IResult<Span, Token> {
    map(
        terminated(
            tuple((
                delimited(
                    tuple((tag_no_case("match"), multispace1)),
                    parse_expression,
                    multispace0
                ),
                many0(delimited(
                    multispace0,
                    alt((
                        parse_match_case,
                        parse_default_case
                    )),
                    multispace0
                ))
            )),
            tag_no_case("end"),
        ),
        |(expr, arms)| {
            Token::Match {
                position: TokenPosition::new(&input),
                expr: Box::from(expr),

                // remove default case
                arms: arms.clone().into_iter().filter(|arm| {
                    match arm {
                        Token::DefaultCase { .. } => false,
                        _ => true
                    }
                }).collect(),

                // find the first default
                default: arms.clone().into_iter().find(|arm| {
                    match arm {
                        Token::DefaultCase { .. } => true,
                        _ => false
                    }
                }).map(|arm| Box::from(arm)),
            }
        },
    )(input)
}

fn parse_match_case(input: Span) -> IResult<Span, Token> {
    map(
        tuple((
            delimited(
                tuple((tag_no_case("case"), multispace1)),
                parse_expression,
                multispace0,
            ),
            delimited(
                tag_no_case("then"),
                parse_function_code_block,
                tag_no_case("end"),
            )
        )),
        |(cond, body)| Token::Case {
            position: TokenPosition::new(&input),
            condition: Box::from(cond),
            body,
        },
    )(input)
}

fn parse_default_case(input: Span) -> IResult<Span, Token> {
    map(
        tuple((
            position,
            preceded(
                tuple((tag_no_case("default"), multispace1)),
                delimited(
                    tag_no_case("then"),
                    parse_function_code_block,
                    tag_no_case("end"),
                )
            ),
        )),
        |(pos, block)| Token::DefaultCase {
            position: TokenPosition::new(&pos),
            body: block,
        },
    )(input)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_if_chain() {
        let (_, tokens) = parse_if_chain(Span::new(r#"if 1 == 2 then
            x = 1
        else if 1 > 3 then
            x = 9
        else
            x = 4
        end"#)).unwrap();

        assert_eq!(tokens,
                   Token::IfChain {
                       position: TokenPosition { line: 1, column: 1 },
                       chain: vec![
                           Token::If {
                               position: TokenPosition { line: 1, column: 1 },
                               condition: Box::from(Token::Eq {
                                   expr1: Box::from(Token::Integer(1)),
                                   expr2: Box::from(Token::Integer(2)),
                               }),
                               body: vec![
                                   Token::Assign {
                                       position: TokenPosition { line: 2, column: 13 },
                                       ident: Box::from(Token::Identifier {
                                           position: TokenPosition { line: 2, column: 13 },
                                           name: String::from("x"),
                                       }),
                                       value: Box::from(Token::Integer(1)),
                                   }
                               ],
                           },
                           Token::If {
                               position: TokenPosition { line: 3, column: 9 },
                               condition: Box::from(Token::Gt {
                                   expr1: Box::from(Token::Integer(1)),
                                   expr2: Box::from(Token::Integer(3)),
                               }),
                               body: vec![
                                   Token::Assign {
                                       position: TokenPosition { line: 4, column: 13 },
                                       ident: Box::from(Token::Identifier {
                                           position: TokenPosition { line: 4, column: 13 },
                                           name: String::from("x"),
                                       }),
                                       value: Box::from(Token::Integer(9)),
                                   }
                               ],
                           },
                           Token::Else {
                               position: TokenPosition { line: 5, column: 9 },
                               body: vec![
                                   Token::Assign {
                                       position: TokenPosition { line: 6, column: 13 },
                                       ident: Box::from(Token::Identifier {
                                           position: TokenPosition { line: 6, column: 13 },
                                           name: String::from("x"),
                                       }),
                                       value: Box::from(Token::Integer(4)),
                                   }
                               ],
                           },
                       ],
                   }
        )
    }

    #[test]
    fn test_parse_if_else() {
        let (_, tokens) = parse_if_chain(Span::new(r#"if 1 == 2 then
            x = 8
        else
            x = 5
        end"#)).unwrap();

        assert_eq!(tokens,
                   Token::IfChain {
                       position: TokenPosition { line: 1, column: 1 },
                       chain: vec![
                           Token::If {
                               position: TokenPosition { line: 1, column: 1 },
                               condition: Box::from(Token::Eq {
                                   expr1: Box::from(Token::Integer(1)),
                                   expr2: Box::from(Token::Integer(2)),
                               }),
                               body: vec![
                                   Token::Assign {
                                       position: TokenPosition { line: 2, column: 13 },
                                       ident: Box::from(Token::Identifier {
                                           position: TokenPosition { line: 2, column: 13 },
                                           name: String::from("x"),
                                       }),
                                       value: Box::from(Token::Integer(8)),
                                   }
                               ],
                           },
                           Token::Else {
                               position: TokenPosition { line: 3, column: 9 },
                               body: vec![
                                   Token::Assign {
                                       position: TokenPosition { line: 4, column: 13 },
                                       ident: Box::from(Token::Identifier {
                                           position: TokenPosition { line: 4, column: 13 },
                                           name: String::from("x"),
                                       }),
                                       value: Box::from(Token::Integer(5)),
                                   }
                               ],
                           },
                       ],
                   }
        )
    }

    #[test]
    fn test_parse_if_statement() {
        let (_, tokens) = parse_if_chain(Span::new(r#"if 1 == 1 then
            var x = 1
        end"#)).unwrap();

        assert_eq!(tokens,
                   Token::IfChain {
                       position: TokenPosition { line: 1, column: 1 },
                       chain: vec![
                           Token::If {
                               position: TokenPosition { line: 1, column: 1 },
                               condition: Box::from(Token::Eq {
                                   expr1: Box::from(Token::Integer(1)),
                                   expr2: Box::from(Token::Integer(1)),
                               }),
                               body: vec![
                                   Token::Variable {
                                       position: TokenPosition { line: 2, column: 13 },
                                       name: String::from("x"),
                                       as_type: None,
                                       value: Some(Box::from(Token::Integer(1))),
                                   }
                               ],
                           },
                       ],
                   }
        )
    }

    #[test]
    fn test_match_statement() {
        let (_, token) = parse_match_statement(Span::new(r#"match a

            case 1 then
                print("one")
            end

            case 2 then
                print("two")
            end

            case 3 then
                print("three")
            end

            default then
                print("other")
            end

        end"#)).unwrap();

        assert_eq!(token, Token::Match {
            position: TokenPosition { line: 1, column: 1 },
            expr: Box::from(Token::Identifier {
                position: TokenPosition { line: 1, column: 7 },
                name: String::from("a"),
            }),
            arms: vec![
                Token::Case {
                    position: TokenPosition { line: 3, column: 13 },
                    condition: Box::from(Token::Integer(1)),
                    body: vec![
                        Token::Call {
                            position: TokenPosition { line: 4, column: 17 },
                            name: Box::from(Token::Identifier {
                                position: TokenPosition { line: 4, column: 17 },
                                name: String::from("print"),
                            }),
                            input: vec![
                                Token::String(String::from("one"))
                            ],
                        }
                    ],
                },
                Token::Case {
                    position: TokenPosition { line: 7, column: 13 },
                    condition: Box::from(Token::Integer(2)),
                    body: vec![
                        Token::Call {
                            position: TokenPosition { line: 8, column: 17 },
                            name: Box::from(Token::Identifier {
                                position: TokenPosition { line: 8, column: 17 },
                                name: String::from("print"),
                            }),
                            input: vec![
                                Token::String(String::from("two"))
                            ],
                        }
                    ],
                },
                Token::Case {
                    position: TokenPosition { line: 11, column: 13 },
                    condition: Box::from(Token::Integer(3)),
                    body: vec![
                        Token::Call {
                            position: TokenPosition { line: 12, column: 17 },
                            name: Box::from(Token::Identifier {
                                position: TokenPosition { line: 12, column: 17 },
                                name: String::from("print"),
                            }),
                            input: vec![
                                Token::String(String::from("three"))
                            ],
                        }
                    ],
                },
            ],
            default: Some(Box::new(Token::DefaultCase {
                position: TokenPosition { line: 15, column: 13 },
                body: vec![
                    Token::Call {
                        position: TokenPosition { line: 16, column: 17 },
                        name: Box::from(Token::Identifier {
                            position: TokenPosition { line: 16, column: 17 },
                            name: String::from("print"),
                        }),
                        input: vec![
                            Token::String(String::from("other"))
                        ],
                    }
                ],
            })),
        })
    }

    #[test]
    fn test_parse_match_case() {
        let (_, token) = parse_match_case(Span::new(r#"case 1 then
            self.a = 1
        end"#)).unwrap();

        assert_eq!(token, Token::Case {
            position: TokenPosition { line: 1, column: 1 },
            condition: Box::from(Token::Integer(1)),
            body: vec![
                Token::Assign {
                    position: TokenPosition { line: 2, column: 13 },
                    ident: Box::new(Token::DotChain {
                        position: TokenPosition { line: 2, column: 13 },
                        start: Box::new(Token::Identifier {
                            position: TokenPosition { line: 2, column: 13 },
                            name: String::from("self"),
                        }),
                        chain: vec![
                            Token::Identifier {
                                position: TokenPosition { line: 2, column: 18 },
                                name: String::from("a"),
                            },
                        ],
                    }),
                    value: Box::from(Token::Integer(1)),
                }
            ],
        })
    }

    #[test]
    fn test_parse_match_default_case() {
        let (_, token) = parse_default_case(Span::new(r#"default then
            self.a = 1
        end"#)).unwrap();

        assert_eq!(token, Token::DefaultCase {
            position: TokenPosition { line: 1, column: 1 },
            body: vec![
                Token::Assign {
                    position: TokenPosition { line: 2, column: 13 },
                    ident: Box::new(Token::DotChain {
                        position: TokenPosition { line: 2, column: 13 },
                        start: Box::new(Token::Identifier {
                            position: TokenPosition { line: 2, column: 13 },
                            name: String::from("self"),
                        }),
                        chain: vec![
                            Token::Identifier {
                                position: TokenPosition { line: 2, column: 18 },
                                name: String::from("a"),
                            },
                        ],
                    }),
                    value: Box::from(Token::Integer(1)),
                }
            ],
        })
    }

}