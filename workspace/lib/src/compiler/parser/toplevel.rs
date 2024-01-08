use crate::compiler::codegen::syntax::{Syntax, TokenPosition};
use crate::compiler::error::{ParserError, ParserErrorType};
use crate::compiler::parser::lexer::Token;
use crate::compiler::parser::Parser;
use crate::parser_error;

impl Parser {
    pub fn parse_script(&mut self) -> Result<(), ParserError> {
        let mut body = vec![];

        while self.lexer.has_more_tokens() {
            let peeked = self.peek_next_token_or_error()?;

            match peeked.token {
                Token::Import => body.extend(self.parse_import()?),
                Token::Enum => body.push(self.parse_enum()?),
                Token::Class => body.push(self.parse_class()?),
                Token::Function => body.push(self.parse_function()?),
                Token::Module => body.push(self.parse_module()?),
                _ => return parser_error!(peeked.cursor, ParserErrorType::UnwantedToken(peeked.token)),
            }
        }

        self.syntax_tree.extend(body);

        Ok(())
    }

    pub(crate) fn parse_module(&mut self) -> Result<Syntax, ParserError> {
        let module_keyword = self.next_token_or_error()?;

        let module_name = self.match_next_token_or_error(Token::Identifier)?;

        let mut body = vec![];

        while self.lexer.has_more_tokens() {
            let peeked = self.peek_next_token_or_error()?;

            match peeked.token {
                Token::Import => body.extend(self.parse_import()?),
                Token::Constant => body.push(self.parse_constant()?),
                Token::Enum => body.push(self.parse_enum()?),
                Token::Class => body.push(self.parse_class()?),
                Token::Function => body.push(self.parse_function()?),
                Token::Module => body.push(self.parse_module()?),
                Token::End => {
                    self.skip_next_token_or_error()?; // skip end
                    break;
                }
                _ => return parser_error!(peeked.cursor, ParserErrorType::UnwantedToken(peeked.token)),
            }
        }

        Ok(Syntax::Module {
            position: TokenPosition {
                line: module_keyword.cursor.line,
                column: module_keyword.cursor.column,
            },
            module_name: Box::new(Syntax::Identifier {
                position: TokenPosition {
                    line: module_name.cursor.line,
                    column: module_name.cursor.column,
                },
                name: module_name.text,
            }),
            body,
        })
    }

    fn parse_constant(&mut self) -> Result<Syntax, ParserError> {
        let constant_keyword = self.next_token_or_error()?;

        let constant_name = self.match_next_token_or_error(Token::Identifier)?;

        let value = if self.peek_next_token_match(Token::SingleEquals) {
            self.skip_next_token_or_error()?;
            let expr = self.parse_expression()?;
            Some(expr)
        } else {
            None
        };

        Ok(Syntax::Constant {
            position: TokenPosition {
                line: constant_keyword.cursor.line,
                column: constant_keyword.cursor.column,
            },
            name: constant_name.text,
            value: Box::new(value.unwrap_or(Syntax::Null)),
        })
    }

    pub fn parse_enum(&mut self) -> Result<Syntax, ParserError> {
        let keyword = self.next_token_or_error()?;

        let identifier = self.next_token_or_error()?;

        let mut items = vec![];

        while self.lexer.has_more_tokens() {
            let token = self.next_token_or_error()?;

            let item = match token.token {
                Token::Identifier => Syntax::Identifier {
                    position: TokenPosition {
                        line: token.cursor.line,
                        column: token.cursor.column,
                    },
                    name: token.text,
                },
                Token::End => break,
                _ => return parser_error!(token.cursor, ParserErrorType::UnexpectedError),
            };

            items.push(item);
        }

        Ok(Syntax::Enum {
            position: TokenPosition {
                line: keyword.cursor.line,
                column: keyword.cursor.column,
            },
            name: Box::new(Syntax::Identifier {
                position: TokenPosition {
                    line: identifier.cursor.line,
                    column: identifier.cursor.column,
                },
                name: identifier.text,
            }),
            items,
        })
    }

    pub fn parse_class(&mut self) -> Result<Syntax, ParserError> {
        let class_keyword = self.next_token_or_error()?;

        let class_name = self.match_next_token_or_error(Token::Identifier)?;

        let mut attributes = vec![];
        let mut methods = vec![];
        let mut constructor = None;

        while self.lexer.has_more_tokens() {
            let peeked = self.peek_next_token_or_error()?;

            match peeked.token {
                Token::Attribute => attributes.push(self.parse_attribute()?),
                Token::Function => methods.push(self.parse_function()?),
                Token::Constructor => constructor = Some(Box::new(self.parse_constructor()?)),
                Token::End => {
                    self.skip_next_token_or_error()?; // skip end
                    break;
                }
                _ => return parser_error!(peeked.cursor, ParserErrorType::UnwantedToken(peeked.token)),
            }
        }

        Ok(Syntax::Class {
            position: TokenPosition {
                line: class_keyword.cursor.line,
                column: class_keyword.cursor.column,
            },
            class_name: Box::from(Syntax::Identifier {
                position: TokenPosition {
                    line: class_name.cursor.line,
                    column: class_name.cursor.column,
                },
                name: class_name.text,
            }),
            attributes,
            constructor,
            methods,
        })
    }

    fn parse_attribute(&mut self) -> Result<Syntax, ParserError> {
        let attribute_kw = self.next_token_or_error()?;
        let attribute_name = self.match_next_token_or_error(Token::Identifier)?;

        let value = if self.peek_next_token_match(Token::SingleEquals) {
            self.skip_next_token_or_error()?;
            let expr = self.parse_expression()?;
            Some(expr)
        } else {
            None
        };

        Ok(Syntax::Attribute {
            position: TokenPosition {
                line: attribute_kw.cursor.line,
                column: attribute_kw.cursor.column,
            },
            name: Box::from(Syntax::Identifier {
                position: TokenPosition {
                    line: attribute_name.cursor.line,
                    column: attribute_name.cursor.column,
                },
                name: attribute_name.text,
            }),
            value: value.map(Box::new),
        })
    }

    pub fn parse_constructor(&mut self) -> Result<Syntax, ParserError> {

        // function keyword
        let function_kw = self.match_next_token_or_error(Token::Constructor)?;

        // function args
        let args = self.parse_function_parameters()?;

        // function body
        let body = self.parse_ended_statement_block()?;

        Ok(Syntax::Constructor {
            position: TokenPosition {
                line: function_kw.cursor.line,
                column: function_kw.cursor.column,
            },
            input: args,
            body,
        })
    }
}

#[cfg(test)]
mod test {
    use crate::compiler::codegen::syntax::Syntax;

    use super::*;

    #[test]
    fn test_parse_enum() {
        let source = r#"
            enum Color
                Red
                Green
                Blue
            end
        "#;

        let mut p = Parser::new(source);

        let r = match p.parse_enum() {
            Ok(r) => r,
            Err(e) => {
                assert!(false, "bad parse: {:?}", e);
                return;
            }
        };

        assert_eq!(r, Syntax::Enum {
            position: TokenPosition { line: 2, column: 13 },
            name: Box::new(Syntax::Identifier {
                position: TokenPosition { line: 2, column: 18 },
                name: String::from("Color"),
            }),
            items: vec![
                Syntax::Identifier {
                    position: TokenPosition { line: 3, column: 17 },
                    name: String::from("Red"),
                },
                Syntax::Identifier {
                    position: TokenPosition { line: 4, column: 17 },
                    name: String::from("Green"),
                },
                Syntax::Identifier {
                    position: TokenPosition { line: 5, column: 17 },
                    name: String::from("Blue"),
                },
            ],
        });
    }

    #[test]
    fn test_parse_class_attributes() {
        let source = r#"
            class Color
                attribute red
                attribute green
                attribute blue
            end
        "#;

        let mut p = Parser::new(source);

        let r = match p.parse_class() {
            Ok(r) => r,
            Err(e) => {
                assert!(false, "bad parse: {:?}", e);
                return;
            }
        };

        assert_eq!(r, Syntax::Class {
            position: TokenPosition { line: 2, column: 13 },
            class_name: Box::new(Syntax::Identifier {
                position: TokenPosition { line: 2, column: 19 },
                name: String::from("Color"),
            }),
            attributes: vec![
                Syntax::Attribute {
                    position: TokenPosition { line: 3, column: 17 },
                    name: Box::new(Syntax::Identifier {
                        position: TokenPosition { line: 3, column: 27 },
                        name: String::from("red"),
                    }),
                    value: None,
                },
                Syntax::Attribute {
                    position: TokenPosition { line: 4, column: 17 },
                    name: Box::new(Syntax::Identifier {
                        position: TokenPosition { line: 4, column: 27 },
                        name: String::from("green"),
                    }),
                    value: None,
                },
                Syntax::Attribute {
                    position: TokenPosition { line: 5, column: 17 },
                    name: Box::new(Syntax::Identifier {
                        position: TokenPosition { line: 5, column: 27 },
                        name: String::from("blue"),
                    }),
                    value: None,
                },
            ],
            constructor: None,
            methods: vec![],
        });
    }

    #[test]
    fn test_class_constructor() {
        let source = r#"
            class Color
                constructor()
                    print("hello world")
                end
            end
        "#;

        let mut p = Parser::new(source);

        let r = match p.parse_class() {
            Ok(r) => r,
            Err(e) => {
                assert!(false, "bad parse: {:?}", e);
                return;
            }
        };

        assert_eq!(r, Syntax::Class {
            position: TokenPosition { line: 2, column: 13 },
            class_name: Box::new(Syntax::Identifier {
                position: TokenPosition { line: 2, column: 19 },
                name: String::from("Color"),
            }),
            attributes: vec![],
            constructor: Some(Box::new(Syntax::Constructor {
                position: TokenPosition { line: 3, column: 17 },
                input: vec![],
                body: vec![
                    Syntax::Call {
                        position: TokenPosition { line: 4, column: 21 },
                        target: Box::new(Syntax::Identifier {
                            position: TokenPosition { line: 4, column: 21 },
                            name: String::from("print"),
                        }),
                        args: vec![
                            Syntax::String(String::from("hello world")),
                        ],
                    },
                ],
            })),
            methods: vec![],
        });
    }

    #[test]
    fn test_class_method() {
        let source = r#"
            class Color
                function print()
                    print("hello world")
                end
            end
        "#;

        let mut p = Parser::new(source);

        let r = match p.parse_class() {
            Ok(r) => r,
            Err(e) => {
                assert!(false, "bad parse: {:?}", e);
                return;
            }
        };

        assert_eq!(r, Syntax::Class {
            position: TokenPosition { line: 2, column: 13 },
            class_name: Box::new(Syntax::Identifier {
                position: TokenPosition { line: 2, column: 19 },
                name: String::from("Color"),
            }),
            attributes: vec![],
            constructor: None,
            methods: vec![
                Syntax::Function {
                    position: TokenPosition { line: 3, column: 17 },
                    function_name: Box::new(Syntax::Identifier {
                        position: TokenPosition { line: 3, column: 26 },
                        name: String::from("print"),
                    }),
                    is_static: false,
                    parameters: vec![],
                    body: vec![
                        Syntax::Call {
                            position: TokenPosition { line: 4, column: 21 },
                            target: Box::new(Syntax::Identifier {
                                position: TokenPosition { line: 4, column: 21 },
                                name: String::from("print"),
                            }),
                            args: vec![
                                Syntax::String(String::from("hello world")),
                            ],
                        },
                    ],
                },
            ],
        });
    }
}