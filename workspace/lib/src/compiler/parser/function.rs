use crate::compiler::codegen::syntax::{Syntax, TokenPosition};
use crate::compiler::error::ParserError;
use crate::compiler::error::ParserErrorType;
use crate::compiler::parser::lexer::Token;
use crate::compiler::parser::Parser;
use crate::parser_error;

impl Parser {
    pub fn parse_function(&mut self) -> Result<Syntax, ParserError> {

        // function keyword
        let function_kw = self.match_next_token_or_error(Token::Function)?;

        // function name
        let ident = self.match_next_token_or_error(Token::Identifier)?;

        // function args
        let parameters = self.parse_function_parameters()?;

        // function body
        let body = self.parse_ended_statement_block()?;

        Ok(Syntax::Function {
            position: TokenPosition {
                line: function_kw.cursor.line,
                column: function_kw.cursor.column,
            },
            function_name: Box::new(Syntax::Identifier {
                position: TokenPosition {
                    line: ident.cursor.line,
                    column: ident.cursor.column,
                },
                name: ident.text,
            }),
            is_static: false,
            parameters,
            body,
        })
    }

    pub fn parse_function_parameters(&mut self) -> Result<Vec<Syntax>, ParserError> {
        let mut args = vec![];

        while self.lexer.has_more_tokens() {
            let matched = self.next_token_or_error()?;

            match matched.token {
                Token::Identifier => args.push(Syntax::Variable {
                    position: TokenPosition {
                        line: matched.cursor.line,
                        column: matched.cursor.column,
                    },
                    name: Box::new(Syntax::Identifier {
                        position: TokenPosition {
                            line: matched.cursor.line,
                            column: matched.cursor.column,
                        },
                        name: matched.text,
                    }),
                    value: None,
                }),
                Token::LeftParenthesis => continue,
                Token::Comma => continue,
                Token::RightParenthesis => break,
                _ => return parser_error!(matched.cursor, ParserErrorType::UnwantedToken(matched.token))
            }
        }

        Ok(args)
    }


    pub fn parse_ended_statement_block(&mut self) -> Result<Vec<Syntax>, ParserError> {
        let mut body = vec![];

        while self.lexer.has_more_tokens() {
            let peeked = self.peek_next_token_or_error()?;

            // if peeked token is end, then break
            if peeked.token == Token::End {
                self.skip_next_token_or_error()?;
                break;
            }

            // parse statement
            match self.match_function_statement()? {
                Syntax::Comment { .. } => continue,
                s => body.push(s)
            }
        }

        Ok(body)
    }

    pub fn match_function_statement(&mut self) -> Result<Syntax, ParserError> {
        let peeked = self.peek_next_token_or_error()?;

        match peeked.token {
            Token::Var => self.parse_variable(),
            Token::If => self.parse_if(),
            Token::Match => self.parse_match(),
            Token::For => self.parse_for_loop(),
            Token::While => self.parse_while_loop(),
            Token::Identifier => self.parse_identifier_statement(),
            Token::Continue => {
                self.skip_next_token_or_error()?;
                Ok(Syntax::Continue {
                    position: TokenPosition {
                        line: peeked.cursor.line,
                        column: peeked.cursor.column,
                    }
                })
            }
            Token::Break => {
                self.skip_next_token_or_error()?;
                Ok(Syntax::Break {
                    position: TokenPosition {
                        line: peeked.cursor.line,
                        column: peeked.cursor.column,
                    }
                })
            }

            // fixme: cannot return new object
            Token::Return => self.parse_return(),

            _ => parser_error!(peeked.cursor, ParserErrorType::UnwantedToken(peeked.token)),
        }
    }

    // fixme: cannot return new object
    fn parse_return(&mut self) -> Result<Syntax, ParserError> {
        let keyword = self.match_next_token_or_error(Token::Return)?;

        let mut expr = None;

        if self.lexer.has_more_tokens() {
            let peeked = self.peek_next_token_or_error()?;

            if Parser::is_expression_token(peeked.token) {
                expr = Some(Box::new(self.parse_expression()?))
            };
        }

        Ok(Syntax::Return {
            position: TokenPosition {
                line: keyword.cursor.line,
                column: keyword.cursor.column,
            },
            expr,
        })
    }
}

mod test {
    use super::*;

    #[test]
    fn test_function_with_args() {
        let source = r#"
            function main(a, b)
                var a = 1
            end
        "#;

        let mut p = Parser::new(source);

        let r = match p.parse_function() {
            Ok(r) => r,
            Err(e) => {
                assert!(false, "bad parse: {:?}", e);
                return;
            }
        };

        assert_eq!(r, Syntax::Function {
            position: TokenPosition { line: 2, column: 13 },
            function_name: Box::new(Syntax::Identifier {
                position: TokenPosition { line: 2, column: 22 },
                name: String::from("main"),
            }),
            is_static: false,
            parameters: vec![
                Syntax::Variable {
                    position: TokenPosition { line: 2, column: 27 },
                    name: Box::new(Syntax::Identifier {
                        position: TokenPosition { line: 2, column: 27 },
                        name: String::from("a"),
                    }),
                    value: None,
                },
                Syntax::Variable {
                    position: TokenPosition { line: 2, column: 30 },
                    name: Box::new(Syntax::Identifier {
                        position: TokenPosition { line: 2, column: 30 },
                        name: String::from("b"),
                    }),
                    value: None,
                },
            ],
            body: vec![
                Syntax::Variable {
                    position: TokenPosition { line: 3, column: 17 },
                    name: Box::new(Syntax::Identifier {
                        position: TokenPosition { line: 3, column: 21 },
                        name: String::from("a"),
                    }),
                    value: Some(Box::new(Syntax::Integer(1))),
                },
            ],
        }
        )
    }

    #[test]
    fn test_parse_return_with_no_value() {
        let source = r#"return"#;

        let mut p = Parser::new(source);

        assert_eq!(p.parse_return(), Ok(Syntax::Return {
            position: TokenPosition {
                line: 1,
                column: 1,
            },
            expr: None,
        }))
    }

    #[test]
    fn test_parse_return_with_value() {
        let source = r#"return true"#;

        let mut p = Parser::new(source);

        assert_eq!(p.parse_return(), Ok(Syntax::Return {
            position: TokenPosition {
                line: 1,
                column: 1,
            },
            expr: Some(Box::new(Syntax::Bool(true))),
        }))
    }

    #[test]
    fn test_script() {
        let mut p = Parser::new(r#"
        function main(argv)
            var x = 1
        end"#);

        let r = match p.parse_function() {
            Ok(r) => r,
            Err(e) => {
                assert!(false, "bad parse: {:?}", e);
                return;
            }
        };

        assert_eq!(r, Syntax::Function {
            position: TokenPosition { line: 2, column: 9 },
            function_name: Box::new(Syntax::Identifier {
                position: TokenPosition { line: 2, column: 18 },
                name: String::from("main"),
            }),
            is_static: false,
            parameters: vec![
                Syntax::Variable {
                    position: TokenPosition { line: 2, column: 23 },
                    name: Box::new(Syntax::Identifier {
                        position: TokenPosition { line: 2, column: 23 },
                        name: String::from("argv"),
                    }),
                    value: None,
                },
            ],
            body: vec![
                Syntax::Variable {
                    position: TokenPosition { line: 3, column: 13 },
                    name: Box::new(Syntax::Identifier {
                        position: TokenPosition { line: 3, column: 17 },
                        name: String::from("x"),
                    }),
                    value: Some(Box::new(Syntax::Integer(1))),
                },
            ],
        })
    }

    #[test]
    fn return_nothing() {
        let mut p = Parser::new(r#"
        function main()
            return
        end"#);

        let r = match p.parse_function() {
            Ok(r) => r,
            Err(e) => {
                assert!(false, "bad parse: {:?}", e);
                return;
            }
        };

        assert_eq!(r, Syntax::Function {
            position: TokenPosition { line: 2, column: 9 },
            function_name: Box::new(Syntax::Identifier {
                position: TokenPosition { line: 2, column: 18 },
                name: String::from("main"),
            }),
            is_static: false,
            parameters: vec![],
            body: vec![
                Syntax::Return {
                    position: TokenPosition { line: 3, column: 13 },
                    expr: None,
                },
            ],
        })
    }

    #[test]
    fn return_something() {
        let mut p = Parser::new(r#"
        function main()
            return true
        end"#);

        let r = match p.parse_function() {
            Ok(r) => r,
            Err(e) => {
                assert!(false, "bad parse: {:?}", e);
                return;
            }
        };

        assert_eq!(r, Syntax::Function {
            position: TokenPosition { line: 2, column: 9 },
            function_name: Box::new(Syntax::Identifier {
                position: TokenPosition { line: 2, column: 18 },
                name: String::from("main"),
            }),
            is_static: false,
            parameters: vec![],
            body: vec![
                Syntax::Return {
                    position: TokenPosition { line: 3, column: 13 },
                    expr: Some(Box::new(Syntax::Bool(true))),
                },
            ],
        })
    }

    #[test]
    fn return_new_object() {
        let mut p = Parser::new(r#"
        function get_object()
            return Object()
        end"#);

        let r = match p.parse_function() {
            Ok(r) => r,
            Err(e) => {
                assert!(false, "bad parse: {:?}", e);
                return;
            }
        };

        assert_eq!(r, Syntax::Function {
            position: TokenPosition { line: 2, column: 9 },
            function_name: Box::new(Syntax::Identifier {
                position: TokenPosition { line: 2, column: 18 },
                name: String::from("get_object"),
            }),
            is_static: false,
            parameters: vec![],
            body: vec![
                Syntax::Return {
                    position: TokenPosition { line: 3, column: 13 },
                    expr: Some(Box::new(Syntax::Call {
                        position: TokenPosition { line: 3, column: 20 },
                        target: Box::new(Syntax::Identifier {
                            position: TokenPosition { line: 3, column: 20 },
                            name: String::from("Object"),
                        }),
                        args: vec![],
                    })),
                },
            ],
        })
    }
}