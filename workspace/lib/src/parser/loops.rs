use nom::branch::alt;
use nom::bytes::complete::{tag, tag_no_case};
use nom::character::complete::{multispace0, multispace1};
use nom::combinator::{map, opt};
use nom::IResult;
use nom::multi::many1;
use nom::sequence::{delimited, preceded, tuple};

use crate::parser::comments::parse_comment;
use crate::parser::dataobjects::{parse_identifier, parse_identifier_chain};
use crate::parser::expressions::parse_expression;
use crate::parser::functions::{parse_call_function, parse_function_return};
use crate::parser::logic::{parse_if_chain, parse_match_statement};
use crate::parser::Span;
use crate::parser::token::{Token, TokenPosition};
use crate::parser::variables::{parse_assignment, parse_variable};

pub fn parse_loop_code_block(input: Span) -> IResult<Span, Vec<Token>> {
    many1(
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
                parse_function_return,
                parse_break,
                parse_continue
            )),
            multispace0)
    )(input)
}

pub fn parse_while_loop(input: Span) -> IResult<Span, Token> {
    map(
        tuple((
            preceded(
                tag_no_case("while"),
                parse_expression,
            ),
            delimited(
                tag_no_case("do"),
                parse_loop_code_block,
                tag_no_case("end"),
            )
        )),
        |(cond, block)| Token::WhileLoop {
            position: TokenPosition::new(&input),
            condition: Box::from(cond),
            body: block,
        },
    )(input)
}

// ((for) (x in v1) (do)) block end
pub fn parse_for_in_loop(input: Span) -> IResult<Span, Token> {
    map(
        tuple((
            preceded(
                tag_no_case("for"),
                delimited(multispace1, parse_identifier, multispace1),
            ),
            preceded(
                tag_no_case("in"),
                parse_expression,
            ),
            delimited(
                tag_no_case("do"),
                parse_loop_code_block,
                tag_no_case("end"),
            )
        )),
        |(ident, target, block)| Token::ForEach {
            position: TokenPosition::new(&input),
            ident: Box::from(ident),
            collection: Box::from(target),
            body: block,
        },
    )(input)
}

// for, (t = start_at) to target step 1, do
pub fn parse_for_to_step(input: Span) -> IResult<Span, Token> {
    map(
        tuple((
            preceded(
                tag_no_case("for"),
                delimited(multispace1, parse_identifier, multispace1),
            ),
            preceded(
                tag("="),
                parse_expression,
            ),
            preceded(
                tag_no_case("to"),
                parse_expression,
            ),
            opt(preceded(
                tag_no_case("step"),
                parse_expression,
            )),
            delimited(
                tag_no_case("do"),
                parse_loop_code_block,
                tag_no_case("end"),
            )
        )),
        |(ident, start, end, step, body)| Token::ForI {
            position: TokenPosition::new(&input),
            ident: Box::from(ident),
            start: Box::from(start),
            step: match step {
                Some(step) => Box::from(step),
                None => Box::from(Token::Integer(1))
            },
            end: Box::from(end),
            body,
        },
    )(input)
}

fn parse_break(input: Span) -> IResult<Span, Token> {
    map(
        tag_no_case("break"),
        |_| Token::Break {
            position: TokenPosition::new(&input)
        },
    )(input)
}

fn parse_continue(input: Span) -> IResult<Span, Token> {
    map(
        tag_no_case("continue"),
        |_| Token::Continue {
            position: TokenPosition::new(&input)
        },
    )(input)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_while_loop() {
        let (_, token) = parse_while_loop(Span::new(r#"while a > 4 do
            print(a)
        end"#)).unwrap();

        assert_eq!(token, Token::WhileLoop {
            position: TokenPosition { line: 1, column: 1 },
            condition: Box::from(Token::Gt {
                expr1: Box::from(Token::Identifier {
                    position: TokenPosition { line: 1, column: 7 },
                    name: String::from("a"),
                }),
                expr2: Box::from(Token::Integer(4)),
            }),
            body: vec![
                Token::Call {
                    position: TokenPosition { line: 2, column: 13 },
                    name: Box::from(Token::Identifier {
                        position: TokenPosition { line: 2, column: 13 },
                        name: String::from("print"),
                    }),
                    input: vec![
                        Token::Identifier {
                            position: TokenPosition { line: 2, column: 19 },
                            name: String::from("a"),
                        }
                    ],
                }
            ],
        })
    }

    #[test]
    fn test_parse_for_in_loop() {
        let (_, token) = parse_for_in_loop(Span::new(r#"for book in books do
            print(book)
        end"#)).unwrap();

        assert_eq!(token, Token::ForEach {
            position: TokenPosition { line: 1, column: 1 },
            ident: Box::from(Token::Identifier {
                position: TokenPosition { line: 1, column: 5 },
                name: String::from("book"),
            }),
            collection: Box::from(Token::Identifier {
                position: TokenPosition { line: 1, column: 13 },
                name: String::from("books"),
            }),
            body: vec![
                Token::Call {
                    position: TokenPosition { line: 2, column: 13 },
                    name: Box::from(Token::Identifier {
                        position: TokenPosition { line: 2, column: 13 },
                        name: String::from("print"),
                    }),
                    input: vec![
                        Token::Identifier {
                            position: TokenPosition { line: 2, column: 19 },
                            name: String::from("book"),
                        }
                    ],
                }
            ],
        })
    }

}