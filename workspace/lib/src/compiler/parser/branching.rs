use crate::compiler::codegen::syntax::{Syntax, TokenPosition};
use crate::compiler::error::{ParserError, ParserErrorType};
use crate::compiler::parser::lexer::Token;
use crate::compiler::parser::Parser;
use crate::parser_error;

impl Parser {
    pub fn parse_if(&mut self) -> Result<Syntax, ParserError> {

        // peek because each if block is parsed by another function
        let keyword = self.peek_next_token_or_error()?;

        // first if block
        let mut if_blocks = vec![self.parse_if_block()?];

        // each if block after that
        while self.lexer.has_more_tokens() {

            // loop until no mor else blocks
            if self.peek_next_token_match(Token::Else) {

                // consume else
                let else_kw = self.next_token_or_error()?;

                // if next token is if, then it's an else if
                if self.peek_next_token_match(Token::If) {
                    if_blocks.push(self.parse_if_block()?)
                } else {
                    let else_body = self.parse_if_body()?;

                    if_blocks.push(Syntax::Else {
                        position: TokenPosition {
                            line: else_kw.cursor.line,
                            column: else_kw.cursor.column,
                        },
                        body: else_body,
                    })
                }
            } else {
                break;
            }
        }

        Ok(Syntax::IfChain {
            position: TokenPosition {
                line: keyword.cursor.line,
                column: keyword.cursor.column,
            },
            arms: if_blocks,
        })
    }

    fn parse_if_block(&mut self) -> Result<Syntax, ParserError> {
        let keyword = self.match_next_token_or_error(Token::If)?;

        // condition
        let condition = self.parse_expression()?;

        // then
        self.skip_matched_token_or_error(Token::Then)?;

        // body
        let body = self.parse_if_body()?;

        Ok(Syntax::If {
            position: TokenPosition {
                line: keyword.cursor.line,
                column: keyword.cursor.column,
            },
            condition: Box::new(condition),
            body,
        })
    }

    fn parse_if_body(&mut self) -> Result<Vec<Syntax>, ParserError> {
        let mut body = vec![];

        while self.lexer.has_more_tokens() {
            let peeked = self.peek_next_token_or_error()?;

            // if peeked token is end, then break
            if peeked.token == Token::End {
                self.skip_next_token_or_error()?; // consume end or else
                break;
            }

            if peeked.token == Token::Else {
                break;
            }

            body.push(self.match_function_statement()?)
        }

        Ok(body)
    }

    pub fn parse_match(&mut self) -> Result<Syntax, ParserError> {
        let keyword = self.match_next_token_or_error(Token::Match)?;
        let mut arms = vec![];
        let mut default = None;

        let expr = self.parse_expression()?;

        while self.lexer.has_more_tokens() {
            let next = self.next_token_or_error()?;

            match next.token {
                Token::Case => {
                    let condition = self.parse_expression()?;
                    self.skip_matched_token_or_error(Token::Then)?;
                    let body = self.parse_if_body()?;
                    arms.push(Syntax::Case {
                        position: TokenPosition {
                            line: next.cursor.line,
                            column: next.cursor.column,
                        },
                        condition: Box::new(condition),
                        body,
                    })
                }
                Token::Default => {
                    self.skip_matched_token_or_error(Token::Then)?;
                    let body = self.parse_if_body()?;
                    default = Some(Syntax::DefaultCase {
                        position: TokenPosition {
                            line: next.cursor.line,
                            column: next.cursor.column,
                        },
                        body,
                    })
                }
                Token::End => {
                    break;
                }
                _ => return parser_error!(next.cursor, ParserErrorType::UnwantedToken(next.token))
            };
        }

        Ok(Syntax::Match {
            position: TokenPosition {
                line: keyword.cursor.line,
                column: keyword.cursor.column,
            },
            expr: Box::new(expr),
            arms,
            default: default.map(Box::new),
        })
    }
}

#[cfg(test)]
mod test {
    use crate::compiler::codegen::syntax::{Syntax, TokenPosition};

    use super::*;

    macro_rules! run_if_test {
        ($source:expr, $expected:expr) => {
            let m = match Parser::new($source).parse_if() {
                Ok(m) => m,
                Err(e) => {
                    println!("{:?}", e);
                    assert!(false, "Expected expression, got error: {:?}", e);
                    return;
                }
            };
            assert_eq!(m, $expected);
        }
    }

    macro_rules! run_match_test {
        ($source:expr, $expected:expr) => {
            let mut p = Parser::new($source);
            let m = match p.parse_match() {
                Ok(m) => m,
                Err(e) => {
                    println!("{:?}", e);
                    assert!(false, "Expected expression, got error: {:?}", e);
                    return;
                }
            };
            assert_eq!(m, $expected);
        }
    }

    #[test]
    fn test_single_if() {
        run_if_test!(
            r#"if count < 99 then
                a = 1
            end"#,
            Syntax::IfChain {
                position: TokenPosition { line: 1, column: 1 },
                arms: vec![Syntax::If {
                    position: TokenPosition { line: 1, column: 1 },
                    condition: Box::new(Syntax::Lt {
                        expr1: Box::new(Syntax::Identifier {
                            position: TokenPosition { line: 1, column: 4 },
                            name: String::from("count"),
                        }),
                        expr2: Box::new(Syntax::Integer(99)),
                    }),
                    body: vec![
                        Syntax::Assign {
                            position: TokenPosition { line: 1, column: 19 },
                            target: Box::new(Syntax::Identifier {
                                position: TokenPosition { line: 2, column: 17 },
                                name: String::from("a"),
                            }),
                            value: Box::new(Syntax::Integer(1)),
                        }
                    ],
                }],
            }
        );
    }

    #[test]
    fn test_if_else() {
        run_if_test!(r#"
            if count > 0 then
                a = 1
            else
                a = 2
            end"#,
            Syntax::IfChain {
                position: TokenPosition { line: 2, column: 13 },
                arms: vec![
                    Syntax::If {
                        position: TokenPosition { line: 2, column: 13 },
                        condition: Box::new(Syntax::Gt {
                            expr1: Box::new(Syntax::Identifier {
                                position: TokenPosition { line: 2, column: 16 },
                                name: String::from("count"),
                            }),
                            expr2: Box::new(Syntax::Integer(0)),
                        }),
                        body: vec![
                            Syntax::Assign {
                                position: TokenPosition { line: 2, column: 30 },
                                target: Box::new(Syntax::Identifier {
                                    position: TokenPosition { line: 3, column: 17 },
                                    name: String::from("a"),
                                }),
                                value: Box::new(Syntax::Integer(1)),
                            }
                        ],
                    },
                    Syntax::Else {
                        position: TokenPosition { line: 4, column: 13 },
                        body: vec![
                            Syntax::Assign {
                                position: TokenPosition { line: 4, column: 17 },
                                target: Box::new(Syntax::Identifier {
                                    position: TokenPosition { line: 5, column: 17 },
                                    name: String::from("a"),
                                }),
                                value: Box::new(Syntax::Integer(2)),
                            }
                        ],
                    }
                ]
            }
        );
    }

    #[test]
    fn test_if_else_if() {
        run_if_test!(r#"
            if count > 0 then
                a = 1
            else if count < 99 then
                a = 2
            end"#,
            Syntax::IfChain {
                position: TokenPosition { line: 2, column: 13 },
                arms: vec![
                    Syntax::If {
                        position: TokenPosition { line: 2, column: 13 },
                        condition: Box::new(Syntax::Gt {
                            expr1: Box::new(Syntax::Identifier {
                                position: TokenPosition { line: 2, column: 16 },
                                name: String::from("count"),
                            }),
                            expr2: Box::new(Syntax::Integer(0)),
                        }),
                        body: vec![
                            Syntax::Assign {
                                position: TokenPosition { line: 2, column: 30 },
                                target: Box::new(Syntax::Identifier {
                                    position: TokenPosition { line: 3, column: 17 },
                                    name: String::from("a"),
                                }),
                                value: Box::new(Syntax::Integer(1)),
                            }
                        ],
                    },
                    Syntax::If {
                        position: TokenPosition { line: 4, column: 18 },
                        condition: Box::new(Syntax::Lt {
                            expr1: Box::new(Syntax::Identifier {
                                position: TokenPosition { line: 4, column: 21 },
                                name: String::from("count"),
                            }),
                            expr2: Box::new(Syntax::Integer(99)),
                        }),
                        body: vec![
                            Syntax::Assign {
                                position: TokenPosition { line: 4, column: 36 },
                                target: Box::new(Syntax::Identifier {
                                    position: TokenPosition { line: 5, column: 17 },
                                    name: String::from("a"),
                                }),
                                value: Box::new(Syntax::Integer(2)),
                            }
                        ],
                    }
                ]
            }
        );
    }

    #[test]
    fn test_if_else_if_else() {
        run_if_test!(r#"
            if count > 0 then
                a = 1
            else if count < 99 then
                a = 2
            else
                a = 3
            end"#,
            Syntax::IfChain {
                position: TokenPosition { line: 2, column: 13 },
                arms: vec![
                    Syntax::If {
                        position: TokenPosition { line: 2, column: 13 },
                        condition: Box::new(Syntax::Gt {
                            expr1: Box::new(Syntax::Identifier {
                                position: TokenPosition { line: 2, column: 16 },
                                name: String::from("count"),
                            }),
                            expr2: Box::new(Syntax::Integer(0)),
                        }),
                        body: vec![
                            Syntax::Assign {
                                position: TokenPosition { line: 2, column: 30 },
                                target: Box::new(Syntax::Identifier {
                                    position: TokenPosition { line: 3, column: 17 },
                                    name: String::from("a"),
                                }),
                                value: Box::new(Syntax::Integer(1)),
                            }
                        ],
                    },
                    Syntax::If {
                        position: TokenPosition { line: 4, column: 18 },
                        condition: Box::new(Syntax::Lt {
                            expr1: Box::new(Syntax::Identifier {
                                position: TokenPosition { line: 4, column: 21 },
                                name: String::from("count"),
                            }),
                            expr2: Box::new(Syntax::Integer(99)),
                        }),
                        body: vec![
                            Syntax::Assign {
                                position: TokenPosition { line: 4, column: 36 },
                                target: Box::new(Syntax::Identifier {
                                    position: TokenPosition { line: 5, column: 17 },
                                    name: String::from("a"),
                                }),
                                value: Box::new(Syntax::Integer(2)),
                            }
                        ],
                    },
                    Syntax::Else {
                        position: TokenPosition { line: 6, column: 13 },
                        body: vec![
                            Syntax::Assign {
                                position: TokenPosition { line: 6, column: 17 },
                                target: Box::new(Syntax::Identifier {
                                    position: TokenPosition { line: 7, column: 17 },
                                    name: String::from("a"),
                                }),
                                value: Box::new(Syntax::Integer(3)),
                            }
                        ],
                    }
                ]
            }
        );
    }

    #[test]
    fn test_parse_match() {
        run_match_test!(r#"
            match count
                case 1 then
                    a = 1
                end
                case 2 then
                    a = 2
                end
                default then
                    a = 3
                end
            end"#,
            Syntax::Match {
                position: TokenPosition { line: 2, column: 13 },
                expr: Box::new(Syntax::Identifier {
                    position: TokenPosition { line: 2, column: 19 },
                    name: String::from("count"),
                }),
                arms: vec![
                    Syntax::Case {
                        position: TokenPosition { line: 3, column: 17 },
                        condition: Box::new(Syntax::Integer(1)),
                        body: vec![
                            Syntax::Assign {
                                position: TokenPosition { line: 3, column: 28 },
                                target: Box::new(Syntax::Identifier {
                                    position: TokenPosition { line: 4, column: 21 },
                                    name: String::from("a"),
                                }),
                                value: Box::new(Syntax::Integer(1)),
                            }
                        ],
                    },
                    Syntax::Case {
                        position: TokenPosition { line: 6, column: 17 },
                        condition: Box::new(Syntax::Integer(2)),
                        body: vec![
                            Syntax::Assign {
                                position: TokenPosition { line: 6, column: 28 },
                                target: Box::new(Syntax::Identifier {
                                    position: TokenPosition { line: 7, column: 21 },
                                    name: String::from("a"),
                                }),
                                value: Box::new(Syntax::Integer(2)),
                            }
                        ],
                    },
                ],
                default: Some(Box::new(Syntax::DefaultCase {
                    position: TokenPosition { line: 9, column: 17 },
                    body: vec![
                        Syntax::Assign {
                            position: TokenPosition { line: 9, column: 29 },
                            target: Box::new(Syntax::Identifier {
                                position: TokenPosition { line: 10, column: 21 },
                                name: String::from("a"),
                            }),
                            value: Box::new(Syntax::Integer(3)),
                        }
                    ],
                })),
            }
        );
    }
}
