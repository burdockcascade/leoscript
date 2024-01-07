use crate::compiler::codegen::syntax::{Syntax, TokenPosition};
use crate::compiler::error::ParserError;
use crate::compiler::parser::lexer::Token;
use crate::compiler::parser::Parser;

impl Parser {
    pub fn parse_variable(&mut self) -> Result<Syntax, ParserError> {
        let keyword = self.match_next_token_or_error(Token::Var)?;

        // get variable name as identifier
        let variable_name = self.match_next_token_or_error(Token::Identifier)?;

        // todo support this?
        // as type
        let as_type = if self.peek_next_token_match(Token::As) {
            // consume as
            self.skip_next_token_or_error()?;

            // get type
            let type_name = self.match_next_token_or_error(Token::Identifier)?;

            Some(type_name.text)
        } else {
            None
        };

        // value is optional
        let v = if self.peek_next_token_match(Token::SingleEquals) {

            // consume equals
            self.skip_next_token_or_error()?;

            // parse expression
            Some(self.parse_expression()?)
        } else {
            None
        };

        // return variable
        Ok(Syntax::Variable {
            position: TokenPosition {
                line: keyword.cursor.line,
                column: keyword.cursor.column,
            },
            name: Box::new(Syntax::Identifier {
                position: TokenPosition {
                    line: variable_name.cursor.line,
                    column: variable_name.cursor.column,
                },
                name: variable_name.text,
            }),
            value: v.map(|v| Box::new(v)),
        })
    }
}

#[cfg(test)]
mod test {
    use crate::compiler::codegen::syntax::{Syntax, TokenPosition};
    use crate::compiler::parser::Parser;

    #[test]
    fn test_parse_variable_with_no_value() {
        let r = match Parser::new("var fruit").parse_variable() {
            Ok(r) => r,
            Err(e) => {
                assert!(false, "bad parse: {:?}", e);
                return;
            }
        };

        assert_eq!(r, Syntax::Variable {
            position: TokenPosition {
                line: 1,
                column: 1,
            },
            name: Box::new(Syntax::Identifier {
                position: TokenPosition {
                    line: 1,
                    column: 5,
                },
                name: "fruit".to_string(),
            }),
            value: None,
        });
    }

    #[test]
    fn test_parse_variable_with_value() {
        let source = "var apples = 3";

        let r = match Parser::new(source).parse_variable() {
            Ok(r) => r,
            Err(e) => {
                assert!(false, "bad parse: {:?}", e);
                return;
            }
        };

        assert_eq!(r, Syntax::Variable {
            position: TokenPosition {
                line: 1,
                column: 1,
            },
            name: Box::new(Syntax::Identifier {
                position: TokenPosition {
                    line: 1,
                    column: 5,
                },
                name: "apples".to_string(),
            }),
            value: Some(Box::new(Syntax::Integer(3))),
        });
    }
}