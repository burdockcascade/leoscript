use nom::branch::alt;
use nom::bytes::complete::{tag, tag_no_case};
use nom::character::complete::{char, multispace0, multispace1};
use nom::combinator::{map, opt};
use nom::IResult;
use nom::multi::many0;
use nom::sequence::{delimited, tuple};

use crate::parser::dataobjects::{parse_identifier, parse_identifier_chain};
use crate::parser::functions::parse_call_function;
use crate::parser::literal::parse_literal;
use crate::parser::Span;
use crate::parser::token::Token;

pub fn parse_expression(input: Span) -> IResult<Span, Token> {
    map(
        tuple((
            parse_and,
            many0(tuple((
                tag_no_case("or"),
                parse_and
            )))
        )),
        |(num1, exprs)| parse_expr_tag(num1, exprs),
    )(input)
}

fn parse_and(input: Span) -> IResult<Span, Token> {
    map(
        tuple((
            parse_expression_stmt,
            many0(tuple((
                tag_no_case("and"),
                parse_expression_stmt
            )))
        )),
        |(num1, exprs)| parse_expr_tag(num1, exprs),
    )(input)
}

fn parse_expression_stmt(input: Span) -> IResult<Span, Token> {
    map(
        tuple((
            opt(delimited(
                multispace0,
                tag_no_case("not"),
                multispace1,
            )),
            delimited(
                multispace0,
                parse_expression2,
                multispace0,
            )
        )),
        |(not, expr)| {
            if not.is_some() {
                Token::Not {
                    expr: Box::from(expr),
                }
            } else {
                expr
            }
        },
    )(input)
}

fn parse_expression2(input: Span) -> IResult<Span, Token> {
    map(
        tuple((
            parse_math_expr,
            many0(tuple((
                delimited(
                    multispace0,
                    alt((tag("=="), tag("!="), tag("<="), tag(">="), tag("<"), tag(">"))),
                    multispace0,
                ),
                parse_math_expr
            )))
        )),
        |(num1, exprs)| parse_expr_tag(num1, exprs),
    )(input)
}

fn parse_math_expr(input: Span) -> IResult<Span, Token> {
    map(
        tuple((
            parse_term,
            many0(tuple((
                delimited(
                    multispace0,
                    alt((tag("+"), tag("-"))),
                    multispace0,
                ),
                parse_term
            )))
        )),
        |(num1, exprs)| parse_expr_tag(num1, exprs),
    )(input)
}

fn parse_term(input: Span) -> IResult<Span, Token> {
    map(
        tuple((
            parse_exponents,
            many0(tuple((
                delimited(
                    multispace0,
                    alt((tag("*"), tag("/"))),
                    multispace0,
                ),
                parse_exponents
            )))
        )),
        |(num1, exprs)| parse_expr_tag(num1, exprs),
    )(input)
}

fn parse_exponents(input: Span) -> IResult<Span, Token> {
    map(
        tuple((
            alt((parse_parentheses, parse_value)),
            many0(tuple((
                delimited(
                    multispace0,
                    tag("^"),
                    multispace0,
                ),
                parse_value
            )))
        )),
        |(num1, exprs)| parse_expr_tag(num1, exprs),
    )(input)
}

fn parse_value(input: Span) -> IResult<Span, Token> {
    alt((
        parse_literal,
        parse_call_function,
        parse_identifier_chain,
        parse_identifier,
    ))(input)
}

fn parse_parentheses(input: Span) -> IResult<Span, Token> {
    delimited(
        multispace0,
        delimited(
            char('('),
            parse_expression,
            char(')'),
        ),
        multispace0,
    )(input)
}

fn parse_expr_tag(expr: Token, rem: Vec<(Span, Token)>) -> Token {
    rem.into_iter().fold(expr, |expr1, (op, expr2)| {
        match *op.fragment() {
            "+" => Token::Add {
                expr1: Box::from(expr1),
                expr2: Box::from(expr2),
            },
            "-" => Token::Sub {
                expr1: Box::from(expr1),
                expr2: Box::from(expr2),
            },
            "*" => Token::Mul {
                expr1: Box::from(expr1),
                expr2: Box::from(expr2),
            },
            "/" => Token::Div {
                expr1: Box::from(expr1),
                expr2: Box::from(expr2),
            },
            "^" => Token::Pow {
                expr1: Box::from(expr1),
                expr2: Box::from(expr2),
            },
            "==" => Token::Eq {
                expr1: Box::from(expr1),
                expr2: Box::from(expr2),
            },
            ">" => Token::Gt {
                expr1: Box::from(expr1),
                expr2: Box::from(expr2),
            },
            ">=" => Token::Ge {
                expr1: Box::from(expr1),
                expr2: Box::from(expr2),
            },
            "<" => Token::Lt {
                expr1: Box::from(expr1),
                expr2: Box::from(expr2),
            },
            "<=" => Token::Le {
                expr1: Box::from(expr1),
                expr2: Box::from(expr2),
            },
            "!=" => Token::Ne {
                expr1: Box::from(expr1),
                expr2: Box::from(expr2),
            },
            "and" => Token::And {
                expr1: Box::from(expr1),
                expr2: Box::from(expr2),
            },
            "or" => Token::Or {
                expr1: Box::from(expr1),
                expr2: Box::from(expr2),
            },
            _ => unimplemented!(),
        }
    })
}

#[cfg(test)]
mod test {
    use crate::parser::token::TokenPosition;

    use super::*;

    #[test]
    fn test_expression_with_identifier() {
        let (span, tokens) = parse_expression(Span::new("a")).unwrap();

        assert_eq!(tokens, Token::Identifier {
            position: TokenPosition { line: 1, column: 1 },
            name: String::from("a"),
        });
    }

    #[test]
    fn test_expression_with_literal_and_identifier() {
        let (_, tokens) = parse_expression(Span::new("b + 34.7")).unwrap();

        assert_eq!(tokens, Token::Add {
            expr1: Box::from(Token::Identifier {
                position: TokenPosition { line: 1, column: 1 },
                name: String::from("b"),
            }),
            expr2: Box::from(Token::Float(34.7)),
        });
    }

    #[test]
    fn test_parse_add_statement() {
        let (_, tokens) = parse_expression(Span::new("12 + 34.7")).unwrap();

        assert_eq!(tokens, Token::Add {
            expr1: Box::from(Token::Integer(12)),
            expr2: Box::from(Token::Float(34.7)),
        });
    }

    #[test]
    fn test_parse_nested_add_sub_statements() {
        let (_, tokens) = parse_expression(Span::new("12 - 34 + 15 - 9")).unwrap();

        assert_eq!(tokens, Token::Sub {
            expr1: Box::from(Token::Add {
                expr1: Box::from(Token::Sub {
                    expr1: Box::from(Token::Integer(12)),
                    expr2: Box::from(Token::Integer(34)),
                }),
                expr2: Box::from(Token::Integer(15)),
            }),
            expr2: Box::from(Token::Integer(9)),
        });
    }

    #[test]
    fn test_parse_multi_level_expression() {
        let (_, tokens) = parse_expression(Span::new("1 * 2 + 3 / 4 ^ 6 == 2")).unwrap();

        assert_eq!(tokens, Token::Eq {
            expr1: Box::from(Token::Add {
                expr1: Box::from(Token::Mul {
                    expr1: Box::from(Token::Integer(1)),
                    expr2: Box::from(Token::Integer(2)),
                }),
                expr2: Box::from(Token::Div {
                    expr1: Box::from(Token::Integer(3)),
                    expr2: Box::from(Token::Pow {
                        expr1: Box::from(Token::Integer(4)),
                        expr2: Box::from(Token::Integer(6)),
                    }),
                }),
            }),
            expr2: Box::from(Token::Integer(2)),
        });
    }

    #[test]
    fn test_parse_expression_with_parantheses() {
        let (_, token) = parse_expression(Span::new("(1 + 2) * 3")).unwrap();

        assert_eq!(token, Token::Mul {
            expr1: Box::from(Token::Add {
                expr1: Box::from(Token::Integer(1)),
                expr2: Box::from(Token::Integer(2)),
            }),
            expr2: Box::from(Token::Integer(3)),
        });
    }

    #[test]
    fn test_parse_expression_equals() {
        let (_, token) = parse_expression(Span::new("1 == 1")).unwrap();

        assert_eq!(token, Token::Eq {
            expr1: Box::from(Token::Integer(1)),
            expr2: Box::from(Token::Integer(1)),
        });
    }

    #[test]
    fn test_expression_with_a_function_call() {
        let (_, tokens) = parse_expression(Span::new("a() + 7")).unwrap();

        assert_eq!(tokens, Token::Add {
            expr1: Box::from(Token::Call {
                position: TokenPosition { line: 1, column: 1 },
                name: Box::from(Token::Identifier {
                    position: TokenPosition { line: 1, column: 1 },
                    name: String::from("a"),
                }),
                input: vec![],
            }),
            expr2: Box::from(Token::Integer(7)),
        });
    }

}