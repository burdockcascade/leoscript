use nom::branch::alt;
use nom::bytes::complete::{tag, tag_no_case};
use nom::character::complete::{alpha1, alphanumeric1, char, multispace0, multispace1};
use nom::combinator::{map, opt, recognize};
use nom::error::ParseError;
use nom::IResult;
use nom::multi::{many0, many1, many_till, separated_list0};
use nom::sequence::{delimited, preceded, terminated, tuple};

use crate::compiler::parser::comments::parse_comment;
use crate::compiler::parser::expressions::parse_expression;
use crate::compiler::parser::functions::{parse_call_function, parse_function, parse_function_code_block};
use crate::compiler::parser::literal::parse_literal;
use crate::compiler::parser::Span;
use crate::compiler::parser::token::{Token, TokenPosition};
use crate::compiler::parser::variables::parse_variable;

const DOUBLE_COLON_OPERATOR: &str = "::";

pub fn parse_identifier(input: Span) -> IResult<Span, Token> {
    map(
        recognize(
            tuple((
                alt((alpha1, tag("_"))),
                many0(alt((alphanumeric1, tag("_")))),
            ))
        ),
        |s: Span| Token::Identifier {
            position: TokenPosition::new(&input),
            name: s.fragment().to_string(),
        },
    )(input)
}

pub fn parse_module(input: Span) -> IResult<Span, Token> {
    map(
        delimited(
            terminated(tag_no_case("module"), multispace0),
            tuple((
                terminated(parse_identifier, multispace0),
                many0(
                    delimited(
                        multispace0,
                        alt((
                            parse_comment,
                            parse_variable,
                            parse_enum,
                            parse_class,
                            parse_constant,
                            parse_module,
                            parse_function
                        )),
                        multispace0,
                    )
                ),
            )),
            tag_no_case("end"),
        ),
        |(name, body)| Token::Module {
            position: TokenPosition::new(&input),
            module_name: Box::from(name),
            body,
        },
    )(input)
}

pub fn parse_enum(input: Span) -> IResult<Span, Token> {
    map(
        preceded(
            terminated(tag_no_case("enum"), multispace0),
            tuple((
                parse_identifier,
                many_till(
                    delimited(
                        multispace0,
                        parse_identifier,
                        multispace0,
                    ),
                    tag_no_case("end"),
                ),
            )),
        ),
        |(name, (items, _))| {
            Token::Enum {
                position: TokenPosition::new(&input),
                name: name.to_string(),
                items,
            }
        },
    )(input)
}

//===========================
// CLASS

pub fn parse_class(input: Span) -> IResult<Span, Token> {
    map(
        delimited(
            terminated(tag_no_case("class"), multispace1),
            tuple((
                terminated(parse_identifier, multispace1),
                many0(
                    delimited(
                        multispace0,
                        alt((
                            parse_comment,
                            parse_class_attribute,
                            parse_class_constructor,
                            parse_function,
                            parse_enum
                        )),
                        multispace0,
                    )
                ),
            )),
            tag_no_case("end"),
        ),
        |(name, body)| Token::Class {
            position: TokenPosition::new(&input),
            class_name: Box::from(name),
            body,
        },
    )(input)
}

fn parse_class_attribute(input: Span) -> IResult<Span, Token> {
    map(
        tuple((
            preceded(
                tuple((tag_no_case("attribute"), multispace1)),
                parse_identifier,
            ),
            opt(preceded(
                delimited(multispace0, tag("="), multispace0),
                alt((
                    parse_literal,
                    parse_new_keyword,
                )),
            ))
        )),
        |(name, value)| Token::Attribute {
            position: TokenPosition::new(&input),
            name: name.to_string(),
            as_type: None, // fixme
            value: value.map_or(None, |token| Some(Box::from(token))),
        },
    )(input)
}

fn parse_class_constructor(input: Span) -> IResult<Span, Token> {
    map(
        terminated(
            tuple((
                delimited(
                    tuple((tag_no_case("constructor"), multispace0)),
                    parse_parameters,
                    multispace0
                ),
                parse_function_code_block
            )),
            tag_no_case("end"),
        ),
        |(params, body)| Token::Constructor {
            position: TokenPosition::new(&input),
            input: params,
            body,
        },
    )(input)
}

fn parse_parameters(input: Span) -> IResult<Span, Vec<Token>> {
    delimited(
        tag("("),
        separated_list0(
            tuple((multispace0, tag(","), multispace0)),
            parse_expression,
        ),
        tag(")"),
    )(input)
}

pub fn parse_new_keyword(input: Span) -> IResult<Span, Token> {
    map(
        preceded(
            tuple((tag_no_case("new"), multispace1)),
            tuple((
                map(
                    tuple((
                        parse_identifier,
                        many0(preceded(tag(DOUBLE_COLON_OPERATOR), parse_identifier))
                    )),
                    |(identifier, items)| {
                        Token::DotChain {
                            position: TokenPosition::new(&input),
                            start: Box::from(identifier),
                            chain: items,
                        }
                    },
                ),
                delimited(
                    tag("("),
                    separated_list0(tuple((multispace0, tag(","), multispace0)), parse_expression),
                    tag(")"),
                )
            )),
        ),
        |(id, args)| Token::NewObject {
            position: TokenPosition::new(&input),
            name: Box::from(id),
            input: args,
        },
    )(input)
}

pub fn parse_constant(input: Span) -> IResult<Span, Token> {
    map(
        tuple((
            preceded(
                tuple((tag_no_case("const"), multispace1)),
                parse_identifier,
            ),
            preceded(
                delimited(multispace0, tag("="), multispace0),
                parse_literal,
            )
        )),
        |(name, value)| Token::Constant {
            position: TokenPosition::new(&input),
            name: name.to_string(),
            value: Box::new(value),
        },
    )(input)
}

// var name [as Integer] = 123


// name = 123


pub fn parse_identifier_chain(input: Span) -> IResult<Span, Token> {
    map(
        tuple((
            parse_identifier,
            many1(
                alt((
                    map(
                        delimited(char('['), parse_expression, char(']')),
                        |item: Token| Token::CollectionIndex(Box::from(item)),
                    ),
                    preceded(
                        char('.'),
                        alt((
                            parse_call_function,
                            parse_identifier
                        )),
                    )
                ))
            )
        )),
        |(identifier, items)| {
            Token::DotChain {
                position: TokenPosition::new(&input),
                start: Box::from(identifier),
                chain: items,
            }
        },
    )(input)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_identifier() {
        let (_, tokens) = parse_identifier(Span::new("hello")).unwrap();

        assert_eq!(tokens, Token::Identifier {
            position: TokenPosition { line: 1, column: 1 },
            name: String::from("hello"),
        })
    }

    #[test]
    fn test_parse_collection_item_with_number() {
        let (_, tokens) = parse_identifier_chain(Span::new("items[6]")).unwrap();

        assert_eq!(tokens, Token::DotChain {
            position: TokenPosition { line: 1, column: 1 },
            start: Box::from(Token::Identifier {
                position: TokenPosition { line: 1, column: 1 },
                name: String::from("items"),
            }),
            chain: vec![
                Token::CollectionIndex(Box::from(Token::Integer(6)))
            ],
        })
    }

    #[test]
    fn test_parse_collection_item_with_string() {
        let (_, tokens) = parse_identifier_chain(Span::new(r#"items["book"]"#)).unwrap();

        assert_eq!(tokens, Token::DotChain {
            position: TokenPosition { line: 1, column: 1 },
            start: Box::from(Token::Identifier {
                position: TokenPosition { line: 1, column: 1 },
                name: String::from("items"),
            }),
            chain: vec![
                Token::CollectionIndex(Box::from(Token::String(String::from("book"))))
            ],
        })
    }

    #[test]
    fn test_parse_call_chain() {
        let (_, tokens) = parse_identifier_chain(Span::new(r#"items[0][3].get_others()[3]["name"].message"#)).unwrap();

        assert_eq!(tokens, Token::DotChain {
            position: TokenPosition { line: 1, column: 1 },
            start: Box::from(Token::Identifier {
                position: TokenPosition { line: 1, column: 1 },
                name: String::from("items"),
            }),
            chain: vec![
                Token::CollectionIndex(Box::from(Token::Integer(0))),
                Token::CollectionIndex(Box::from(Token::Integer(3))),
                Token::Call {
                    position: TokenPosition { line: 1, column: 13 },
                    name: Box::from(Token::Identifier {
                        position: TokenPosition { line: 1, column: 13 },
                        name: String::from("get_others"),
                    }),
                    input: vec![],
                },
                Token::CollectionIndex(Box::from(Token::Integer(3))),
                Token::CollectionIndex(Box::from(Token::String(String::from("name")))),
                Token::Identifier {
                    position: TokenPosition { line: 1, column: 37 },
                    name: String::from("message"),
                },
            ],
        });
    }

    #[test]
    fn test_parse_class_method() {
        let (_, tokens) = parse_identifier_chain(Span::new("items.get_message()")).unwrap();

        assert_eq!(tokens, Token::DotChain {
            position: TokenPosition { line: 1, column: 1 },
            start: Box::from(Token::Identifier {
                position: TokenPosition { line: 1, column: 1 },
                name: String::from("items"),
            }),
            chain: vec![
                Token::Call {
                    position: TokenPosition { line: 1, column: 7 },
                    name: Box::from(Token::Identifier {
                        position: TokenPosition { line: 1, column: 7 },
                        name: String::from("get_message"),
                    }),
                    input: vec![],
                }
            ],
        })
    }

    #[test]
    fn test_parse_class_method_with_params() {
        let (_, tokens) = parse_identifier_chain(Span::new("items.get_message(2)")).unwrap();

        assert_eq!(tokens, Token::DotChain {
            position: TokenPosition { line: 1, column: 1 },
            start: Box::from(Token::Identifier {
                position: TokenPosition { line: 1, column: 1 },
                name: String::from("items"),
            }),
            chain: vec![
                Token::Call {
                    position: TokenPosition { line: 1, column: 7 },
                    name: Box::from(Token::Identifier {
                        position: TokenPosition { line: 1, column: 7 },
                        name: String::from("get_message"),
                    }),
                    input: vec![
                        Token::Integer(2)
                    ],
                }
            ],
        })
    }

    #[test]
    fn test_parse_class_field() {
        let (_, tokens) = parse_identifier_chain(Span::new("items.length")).unwrap();

        assert_eq!(tokens, Token::DotChain {
            position: TokenPosition { line: 1, column: 1 },
            start: Box::from(Token::Identifier {
                position: TokenPosition { line: 1, column: 1 },
                name: String::from("items"),
            }),
            chain: vec![
                Token::Identifier {
                    position: TokenPosition { line: 1, column: 7 },
                    name: String::from("length"),
                }
            ],
        })
    }

    #[test]
    fn test_constant_declaration() {
        let (_, tokens) = parse_constant(Span::new(r#"const PI = 3.14"#)).unwrap();

        assert_eq!(tokens, Token::Constant {
            position: TokenPosition { line: 1, column: 1 },
            name: String::from("PI"),
            value: Box::from(Token::Float(3.14)),
        })
    }

    #[test]
    fn test_parse_class() {
        let (_, tokens) = parse_class(Span::new(r#"class Book

            attribute name
            attribute pages = 0
            attribute author = "unknown"

            constructor(name, pages, author)
                self.name = name
                self.pages = pages
                self.author = author
            end

            function get_name()
                return self.name
            end

        end"#)).unwrap();

        assert_eq!(tokens, Token::Class {
            position: TokenPosition { line: 1, column: 1 },
            class_name: Box::from(Token::Identifier {
                position: TokenPosition { line: 1, column: 7 },
                name: String::from("Book"),
            }),
            body: vec![
                Token::Attribute {
                    position: TokenPosition { line: 3, column: 13 },
                    name: String::from("name"),
                    as_type: None,
                    value: None,
                },
                Token::Attribute {
                    position: TokenPosition { line: 4, column: 13 },
                    name: String::from("pages"),
                    as_type: None,
                    value: Some(Box::from(Token::Integer(0))),
                },
                Token::Attribute {
                    position: TokenPosition { line: 5, column: 13 },
                    name: String::from("author"),
                    as_type: None,
                    value: Some(Box::from(Token::String(String::from("unknown")))),
                },
                Token::Constructor {
                    position: TokenPosition { line: 7, column: 13 },
                    input: vec![
                        Token::Identifier {
                            position: TokenPosition { line: 7, column: 25 },
                            name: String::from("name"),
                        },
                        Token::Identifier {
                            position: TokenPosition { line: 7, column: 31 },
                            name: String::from("pages"),
                        },
                        Token::Identifier {
                            position: TokenPosition { line: 7, column: 38 },
                            name: String::from("author"),
                        },
                    ],
                    body: vec![
                        Token::Assign {
                            position: TokenPosition { line: 8, column: 17 },
                            ident: Box::from(Token::DotChain {
                                position: TokenPosition { line: 8, column: 17 },
                                start: Box::from(Token::Identifier {
                                    position: TokenPosition { line: 8, column: 17 },
                                    name: String::from("self"),
                                }),
                                chain: vec![
                                    Token::Identifier {
                                        position: TokenPosition { line: 8, column: 22 },
                                        name: String::from("name"),
                                    },
                                ],
                            }),
                            value: Box::from(Token::Identifier {
                                position: TokenPosition { line: 8, column: 29 },
                                name: String::from("name"),
                            }),
                        },
                        Token::Assign {
                            position: TokenPosition { line: 9, column: 17 },
                            ident: Box::from(Token::DotChain {
                                position: TokenPosition { line: 9, column: 17 },
                                start: Box::from(Token::Identifier {
                                    position: TokenPosition { line: 9, column: 17 },
                                    name: String::from("self"),
                                }),
                                chain: vec![
                                    Token::Identifier {
                                        position: TokenPosition { line: 9, column: 22 },
                                        name: String::from("pages"),
                                    },
                                ],
                            }),
                            value: Box::from(Token::Identifier {
                                position: TokenPosition { line: 9, column: 30 },
                                name: String::from("pages"),
                            }),
                        },
                        Token::Assign {
                            position: TokenPosition { line: 10, column: 17 },
                            ident: Box::from(Token::DotChain {
                                position: TokenPosition { line: 10, column: 17 },
                                start: Box::from(Token::Identifier {
                                    position: TokenPosition { line: 10, column: 17 },
                                    name: String::from("self"),
                                }),
                                chain: vec![
                                    Token::Identifier {
                                        position: TokenPosition { line: 10, column: 22 },
                                        name: String::from("author"),
                                    },
                                ],
                            }),
                            value: Box::from(Token::Identifier {
                                position: TokenPosition { line: 10, column: 31 },
                                name: String::from("author"),
                            }),
                        },
                    ],
                },
                Token::Function {
                    position: TokenPosition { line: 13, column: 13 },
                    function_name: Box::from(Token::Identifier {
                        position: TokenPosition { line: 13, column: 22 },
                        name: String::from("get_name"),
                    }),
                    is_static: false,
                    scope: None,
                    return_type: None,
                    input: vec![],
                    body: vec![
                        Token::Return {
                            position: TokenPosition { line: 14, column: 17 },
                            expr: Some(Box::from(Token::DotChain {
                                position: TokenPosition { line: 14, column: 24 },
                                start: Box::from(Token::Identifier {
                                    position: TokenPosition { line: 14, column: 24 },
                                    name: String::from("self"),
                                }),
                                chain: vec![
                                    Token::Identifier {
                                        position: TokenPosition { line: 14, column: 29 },
                                        name: String::from("name"),
                                    },
                                ],
                            })),
                        },
                    ],
                },
            ],
        })
    }

    #[test]
    fn test_new_cass_construct_with_parameters() {
        let (_, tokens) = parse_new_keyword(Span::new(r#"new Book("hello", 123)"#)).unwrap();

        assert_eq!(tokens, Token::NewObject {
            position: TokenPosition { line: 1, column: 1 },
            name: Box::from(Token::DotChain {
                position: TokenPosition { line: 1, column: 1 },
                start: Box::from(Token::Identifier {
                    position: TokenPosition { line: 1, column: 5 },
                    name: String::from("Book"),
                }),
                chain: vec![],
            }),
            input: vec![
                Token::String(String::from("hello")),
                Token::Integer(123),
            ],
        })
    }

    #[test]
    fn test_new_cass_construct_from_module() {
        let (_, tokens) = parse_new_keyword(Span::new(r#"new Math::Vector2(12, 64)"#)).unwrap();

        assert_eq!(tokens, Token::NewObject {
            position: TokenPosition { line: 1, column: 1 },
            name: Box::from(Token::DotChain {
                position: TokenPosition { line: 1, column: 1 },
                start: Box::from(Token::Identifier {
                    position: TokenPosition { line: 1, column: 5 },
                    name: String::from("Math"),
                }),
                chain: vec![
                    Token::Identifier {
                        position: TokenPosition { line: 1, column: 11 },
                        name: String::from("Vector2"),
                    },
                ],
            }),
            input: vec![
                Token::Integer(12),
                Token::Integer(64),
            ],
        })
    }


    #[test]
    fn test_parse_enum() {
        let (_, tokens) = parse_enum(Span::new(r#"enum Book
            Book
            Magazine
            Newspaper
        end"#)).unwrap();

        assert_eq!(tokens, Token::Enum {
            position: TokenPosition { line: 1, column: 1 },
            name: String::from("Book"),
            items: vec![
                Token::Identifier {
                    position: TokenPosition { line: 2, column: 13 },
                    name: String::from("Book"),
                },
                Token::Identifier {
                    position: TokenPosition { line: 3, column: 13 },
                    name: String::from("Magazine"),
                },
                Token::Identifier {
                    position: TokenPosition { line: 4, column: 13 },
                    name: String::from("Newspaper"),
                },
            ],
        })
    }

}