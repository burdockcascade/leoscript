use crate::compiler::codegen::syntax::{Syntax, TokenPosition};
use crate::compiler::error::{ParserError, ParserErrorType};
use crate::compiler::parser::lexer::Token;
use crate::compiler::parser::Parser;
use crate::parser_error;

impl Parser {
    pub fn parse_while_loop(&mut self) -> Result<Syntax, ParserError> {
        let keyword = self.match_next_token_or_error(Token::While)?;

        // condition
        let condition = self.parse_expression()?;

        // do
        self.skip_matched_token_or_error(Token::Do)?;

        // body
        let body = self.parse_loop_end_statement_block()?;

        Ok(Syntax::WhileLoop {
            position: TokenPosition {
                line: keyword.cursor.line,
                column: keyword.cursor.column,
            },
            condition: Box::new(condition),
            body,
        })
    }

    pub fn parse_for_loop(&mut self) -> Result<Syntax, ParserError> {
        let keyword = self.match_next_token_or_error(Token::For)?;

        // variable
        let variable = self.match_next_token_or_error(Token::Identifier)?;

        let kw = self.peek_next_token_or_error()?;

        let iterable;

        match kw.token {
            Token::In => {
                self.skip_next_token_or_error()?;

                // iterable
                iterable = self.parse_expression()?;
            }
            Token::SingleEquals => {
                self.skip_next_token_or_error()?;

                // start
                let start = self.parse_expression()?;

                // to
                self.skip_matched_token_or_error(Token::To)?;

                // end
                let end = self.parse_expression()?;

                // step
                let step = if self.peek_next_token_match(Token::Step) {
                    self.skip_matched_token_or_error(Token::Step)?;
                    Some(Box::new(self.parse_expression()?))
                } else {
                    None
                };

                iterable = Syntax::Range {
                    start: Box::new(start),
                    end: Box::new(end),
                    step,
                }
            }
            _ => return parser_error!(kw.cursor, ParserErrorType::UnwantedToken(kw.token))
        }

        // do
        self.skip_matched_token_or_error(Token::Do)?;

        // body
        let body = self.parse_loop_end_statement_block()?;

        Ok(Syntax::ForEach {
            position: TokenPosition {
                line: keyword.cursor.line,
                column: keyword.cursor.column,
            },
            ident: Box::new(Syntax::Identifier {
                position: TokenPosition {
                    line: variable.cursor.line,
                    column: variable.cursor.column,
                },
                name: variable.text,
            }),
            collection: Box::new(iterable),
            body,
        })
    }

    pub fn parse_loop_end_statement_block(&mut self) -> Result<Vec<Syntax>, ParserError> {
        let mut body = vec![];

        while self.lexer.has_more_tokens() {
            let peeked = self.peek_next_token_or_error()?;

            let s = match peeked.token {
                Token::End => {
                    self.skip_next_token_or_error()?;
                    break;
                }
                _ => self.match_function_statement()?
            };

            body.push(s);
        }

        Ok(body)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_while() {
        let mut p = Parser::new(r#"while count > 0 do end"#);
        let m = match p.parse_while_loop() {
            Ok(m) => m,
            Err(e) => {
                assert!(false, "Expected expression, got error: {:?}", e);
                return;
            }
        };

        assert_eq!(m, Syntax::WhileLoop {
            position: TokenPosition { line: 1, column: 1 },
            condition: Box::new(Syntax::Gt {
                expr1: Box::new(Syntax::Identifier {
                    position: TokenPosition { line: 1, column: 7 },
                    name: String::from("count"),
                }),
                expr2: Box::new(Syntax::Integer(0)),
            }),
            body: vec![],
        }
        );
    }

    #[test]
    fn test_while_with_break() {
        let mut p = Parser::new(r#"while count > 0 do break end"#);
        let m = match p.parse_while_loop() {
            Ok(m) => m,
            Err(e) => {
                assert!(false, "Expected expression, got error: {:?}", e);
                return;
            }
        };

        assert_eq!(m, Syntax::WhileLoop {
            position: TokenPosition { line: 1, column: 1 },
            condition: Box::new(Syntax::Gt {
                expr1: Box::new(Syntax::Identifier {
                    position: TokenPosition { line: 1, column: 7 },
                    name: String::from("count"),
                }),
                expr2: Box::new(Syntax::Integer(0)),
            }),
            body: vec![
                Syntax::Break {
                    position: TokenPosition { line: 1, column: 20 },
                }
            ],
        }
        );
    }

    #[test]
    fn test_while_with_continue() {
        let mut p = Parser::new(r#"while count > 0 do continue end"#);
        let m = match p.parse_while_loop() {
            Ok(m) => m,
            Err(e) => {
                assert!(false, "Expected expression, got error: {:?}", e);
                return;
            }
        };

        assert_eq!(m, Syntax::WhileLoop {
            position: TokenPosition { line: 1, column: 1 },
            condition: Box::new(Syntax::Gt {
                expr1: Box::new(Syntax::Identifier {
                    position: TokenPosition { line: 1, column: 7 },
                    name: String::from("count"),
                }),
                expr2: Box::new(Syntax::Integer(0)),
            }),
            body: vec![
                Syntax::Continue {
                    position: TokenPosition { line: 1, column: 20 },
                }
            ],
        }
        );
    }

    #[test]
    fn test_for_in() {
        let mut p = Parser::new(r#"for book in books do end"#);
        let m = match p.parse_for_loop() {
            Ok(m) => m,
            Err(e) => {
                assert!(false, "Expected expression, got error: {:?}", e);
                return;
            }
        };

        assert_eq!(m, Syntax::ForEach {
            position: TokenPosition { line: 1, column: 1 },
            ident: Box::new(Syntax::Identifier {
                position: TokenPosition { line: 1, column: 5 },
                name: String::from("book"),
            }),
            collection: Box::new(Syntax::Identifier {
                position: TokenPosition { line: 1, column: 13 },
                name: String::from("books"),
            }),
            body: vec![],
        });
    }

    #[test]
    fn test_for_to() {
        let mut p = Parser::new(r#"for x = 1 to 100 do end"#);
        let m = match p.parse_for_loop() {
            Ok(m) => m,
            Err(e) => {
                assert!(false, "Expected expression, got error: {:?}", e);
                return;
            }
        };

        assert_eq!(m, Syntax::ForEach {
            position: TokenPosition { line: 1, column: 1 },
            ident: Box::new(Syntax::Identifier {
                position: TokenPosition { line: 1, column: 5 },
                name: String::from("x"),
            }),
            collection: Box::new(Syntax::Range {
                start: Box::new(Syntax::Integer(1)),
                end: Box::new(Syntax::Integer(100)),
                step: None,
            }),
            body: vec![],
        });
    }

    #[test]
    fn test_for_to_with_step() {
        let mut p = Parser::new(r#"for x = 1 to 100 step 2 do end"#);
        let m = match p.parse_for_loop() {
            Ok(m) => m,
            Err(e) => {
                assert!(false, "Expected expression, got error: {:?}", e);
                return;
            }
        };

        assert_eq!(m, Syntax::ForEach {
            position: TokenPosition { line: 1, column: 1 },
            ident: Box::new(Syntax::Identifier {
                position: TokenPosition { line: 1, column: 5 },
                name: String::from("x"),
            }),
            collection: Box::new(Syntax::Range {
                start: Box::new(Syntax::Integer(1)),
                end: Box::new(Syntax::Integer(100)),
                step: Some(Box::new(Syntax::Integer(2))),
            }),
            body: vec![],
        });
    }
}