use crate::compiler::codegen::syntax::{Syntax, TokenPosition};
use crate::compiler::error::ParserError;
use crate::compiler::parser::lexer::Token;
use crate::compiler::parser::Parser;

impl Parser {
    pub fn parse_identifier_statement(&mut self) -> Result<Syntax, ParserError> {
        let position = self.lexer.get_cursor();

        let expr = self.parse_expression()?;

        match expr {
            // nothing will come after this
            Syntax::Call { .. } => return Ok(expr),
            _ => {}
        }

        let peeked = self.peek_next_token_or_error()?;

        match peeked.token {
            Token::SingleEquals => {
                // consume equals
                self.skip_matched_token_or_error(Token::SingleEquals)?;

                // parse expression
                let value = self.parse_expression()?;

                // return variable
                Ok(Syntax::Assign {
                    position: TokenPosition {
                        line: position.line,
                        column: position.column,
                    },
                    target: Box::new(expr),
                    value: Box::new(value),
                })
            }
            _ => Ok(expr)
        }
    }
}

mod test {
    use crate::compiler::codegen::syntax::{Syntax, TokenPosition};
    use crate::compiler::parser::Parser;

    #[test]
    fn test_function_call() {
        let source = r#"print("hello world")"#;

        let mut p = Parser::new(source);
        let result = p.parse_identifier_statement();

        assert_eq!(result, Ok(Syntax::Call {
            position: TokenPosition {
                line: 1,
                column: 1,
            },
            target: Box::new(Syntax::Identifier {
                position: TokenPosition {
                    line: 1,
                    column: 1,
                },
                name: "print".to_string(),
            }),
            args: vec![
                Syntax::String("hello world".to_string())
            ],
        }));
    }

    #[test]
    fn test_assignment_single_identifier() {
        let source = "teacups = 2";

        let mut p = Parser::new(source);

        let result = p.parse_identifier_statement();

        assert_eq!(result, Ok(Syntax::Assign {
            position: TokenPosition {
                line: 1,
                column: 1,
            },
            target: Box::new(Syntax::Identifier {
                position: TokenPosition {
                    line: 1,
                    column: 1,
                },
                name: "teacups".to_string(),
            }),
            value: Box::new(Syntax::Integer(2)),
        }));
    }

    #[test]
    fn test_assignment_to_array_access() {
        let source = r#"library.authors[234] = "Mr Smith""#;

        let mut p = Parser::new(source);
        let result = p.parse_identifier_statement();

        assert_eq!(result, Ok(Syntax::Assign {
            position: TokenPosition { line: 1, column: 1 },
            target: Box::from(Syntax::ArrayAccess {
                position: TokenPosition { line: 1, column: 17 },
                target: Box::from(Syntax::MemberAccess {
                    position: TokenPosition { line: 1, column: 9 },
                    target: Box::from(Syntax::Identifier {
                        position: TokenPosition { line: 1, column: 1 },
                        name: String::from("library"),
                    }),
                    index: Box::from(Syntax::Identifier {
                        position: TokenPosition { line: 1, column: 9 },
                        name: String::from("authors"),
                    }),
                }),
                index: Box::new(Syntax::Integer(234)),
            }),
            value: Box::new(Syntax::String("Mr Smith".to_string())),
        }));
    }

    #[test]
    fn test_assignment_to_expression() {
        let source = r#"library.books[234].title = "Cyber Cow""#;

        let mut p = Parser::new(source);
        let result = p.parse_identifier_statement();

        assert_eq!(result, Ok(Syntax::Assign {
            position: TokenPosition { line: 1, column: 1 },
            target: Box::from(Syntax::MemberAccess {
                position: TokenPosition { line: 1, column: 20 },
                target: Box::from(Syntax::ArrayAccess {
                    position: TokenPosition { line: 1, column: 15 },
                    target: Box::from(Syntax::MemberAccess {
                        position: TokenPosition { line: 1, column: 9 },
                        target: Box::from(Syntax::Identifier {
                            position: TokenPosition { line: 1, column: 1 },
                            name: String::from("library"),
                        }),
                        index: Box::from(Syntax::Identifier {
                            position: TokenPosition { line: 1, column: 9 },
                            name: String::from("books"),
                        }),
                    }),
                    index: Box::new(Syntax::Integer(234)),
                }),
                index: Box::from(Syntax::Identifier {
                    position: TokenPosition { line: 1, column: 20 },
                    name: String::from("title"),
                }),
            }),
            value: Box::new(Syntax::String("Cyber Cow".to_string())),
        }));
    }
}