use std::collections::HashMap;

use crate::compiler::codegen::syntax::Syntax;
use crate::compiler::codegen::syntax::TokenPosition;
use crate::compiler::error::{ParserError, ParserErrorType};
use crate::compiler::parser::lexer::lexer::MatchedToken;
use crate::compiler::parser::lexer::Token;
use crate::compiler::parser::Parser;
use crate::parse_error;

impl Parser {
    // entry point into expression parsing
    pub fn parse_expression(&mut self) -> Result<Syntax, ParserError> {
        self.parse_or()
    }

    // OR operator
    fn parse_or(&mut self) -> Result<Syntax, ParserError> {
        let mut expr = self.parse_and()?;

        while self.lexer.has_more_tokens() {
            let peek = self.peek_next_token_or_error()?;

            match peek.token {
                Token::Or => {
                    self.skip_next_token_or_error()?;
                    expr = Syntax::Or {
                        expr1: Box::new(expr),
                        expr2: Box::new(self.parse_and()?),
                    };
                }
                _ => break
            }
        }

        Ok(expr)
    }

    // AND operator
    fn parse_and(&mut self) -> Result<Syntax, ParserError> {
        let mut expr = self.parse_compare()?;

        while self.lexer.has_more_tokens() {
            let peek = self.peek_next_token_or_error()?;

            match peek.token {
                Token::And => {
                    self.skip_next_token_or_error()?;
                    expr = Syntax::And {
                        expr1: Box::new(expr),
                        expr2: Box::new(self.parse_compare()?),
                    };
                }
                _ => break
            }
        }

        Ok(expr)
    }

    // comparison operators
    fn parse_compare(&mut self) -> Result<Syntax, ParserError> {
        let mut expr = self.parse_math()?;

        while self.lexer.has_more_tokens() {
            let peek = self.peek_next_token_or_error()?;

            match peek.token {
                Token::DoubleEquals => {
                    self.skip_next_token_or_error()?;
                    expr = Syntax::Eq {
                        expr1: Box::new(expr),
                        expr2: Box::new(self.parse_math()?),
                    };
                }
                Token::NotEquals => {
                    self.skip_next_token_or_error()?;
                    expr = Syntax::Ne {
                        expr1: Box::new(expr),
                        expr2: Box::new(self.parse_math()?),
                    };
                }
                Token::Lt => {
                    self.skip_next_token_or_error()?;
                    expr = Syntax::Lt {
                        expr1: Box::new(expr),
                        expr2: Box::new(self.parse_math()?),
                    };
                }
                Token::Le => {
                    self.skip_next_token_or_error()?;
                    expr = Syntax::Le {
                        expr1: Box::new(expr),
                        expr2: Box::new(self.parse_math()?),
                    };
                }
                Token::Gt => {
                    self.skip_next_token_or_error()?;
                    expr = Syntax::Gt {
                        expr1: Box::new(expr),
                        expr2: Box::new(self.parse_math()?),
                    };
                }
                Token::Ge => {
                    self.skip_next_token_or_error()?;
                    expr = Syntax::Ge {
                        expr1: Box::new(expr),
                        expr2: Box::new(self.parse_math()?),
                    };
                }
                _ => break
            }
        }

        Ok(expr)
    }

    // math operators
    fn parse_math(&mut self) -> Result<Syntax, ParserError> {
        let mut expr = self.parse_term()?;

        while self.lexer.has_more_tokens() {
            let peek = self.peek_next_token_or_error()?;

            match peek.token {
                Token::Plus => {
                    self.skip_next_token_or_error()?;
                    expr = Syntax::Add {
                        expr1: Box::new(expr),
                        expr2: Box::new(self.parse_term()?),
                    };
                }
                Token::Minus => {
                    self.skip_next_token_or_error()?;
                    expr = Syntax::Sub {
                        expr1: Box::new(expr),
                        expr2: Box::new(self.parse_term()?),
                    };
                }
                _ => break
            }
        }

        Ok(expr)
    }

    // term operators
    fn parse_term(&mut self) -> Result<Syntax, ParserError> {
        let mut expr = self.parse_pow()?;

        while self.lexer.has_more_tokens() {
            let peek = self.peek_next_token_or_error()?;

            match peek.token {
                Token::Mul => {
                    self.skip_next_token_or_error()?;
                    expr = Syntax::Mul {
                        expr1: Box::new(expr),
                        expr2: Box::new(self.parse_pow()?),
                    };
                }
                Token::Div => {
                    self.skip_next_token_or_error()?;
                    expr = Syntax::Div {
                        expr1: Box::new(expr),
                        expr2: Box::new(self.parse_pow()?),
                    };
                }
                _ => break
            }
        }

        Ok(expr)
    }

    // power operator
    fn parse_pow(&mut self) -> Result<Syntax, ParserError> {
        let mut expr = self.parse_not()?;

        while self.lexer.has_more_tokens() {
            let peek = self.peek_next_token_or_error()?;

            match peek.token {
                Token::Pow => {
                    self.skip_next_token_or_error()?;
                    expr = Syntax::Pow {
                        expr1: Box::new(expr),
                        expr2: Box::new(self.parse_not()?),
                    };
                }
                _ => break
            }
        }

        Ok(expr)
    }

    // not operator
    fn parse_not(&mut self) -> Result<Syntax, ParserError> {
        let peek = self.peek_next_token_or_error()?;

        match peek.token {
            Token::Not => {
                self.skip_next_token_or_error()?;
                Ok(Syntax::Not {
                    expr: Box::new(self.parse_expression()?),
                })
            }
            _ => self.parse_data_access()
        }
    }

    fn parse_data_access(&mut self) -> Result<Syntax, ParserError> {
        let mut position = self.peek_next_token_or_error()?;

        let mut expr = self.parse_primary()?;

        while self.lexer.has_more_tokens() {
            let peek = self.peek_next_token_or_error()?;

            match peek.token {
                Token::LeftParenthesis => {
                    expr = Syntax::Call {
                        position: TokenPosition {
                            line: position.cursor.line,
                            column: position.cursor.column,
                        },
                        target: Box::from(expr),
                        args: self.parse_arguments()?,
                    }
                }
                Token::LeftSquareBracket => {
                    self.skip_next_token_or_error()?;
                    position = self.peek_next_token_or_error()?;
                    expr = Syntax::ArrayAccess {
                        position: TokenPosition {
                            line: position.cursor.line,
                            column: position.cursor.column,
                        },
                        target: Box::from(expr),
                        index: Box::from(self.parse_array_access()?),
                    };
                }
                Token::Dot => {
                    self.skip_next_token_or_error()?;
                    position = self.peek_next_token_or_error()?;
                    expr = Syntax::MemberAccess {
                        position: TokenPosition {
                            line: position.cursor.line,
                            column: position.cursor.column,
                        },
                        target: Box::from(expr),
                        index: {
                            let primary = self.parse_primary()?;
                            match primary {
                                Syntax::Identifier { .. } => Box::from(primary),
                                _ => return parse_error!(position.cursor, ParserErrorType::InvalidMemberAccess)
                            }
                        },
                    }
                }
                Token::DoubleColon => {
                    self.skip_next_token_or_error()?;
                    position = self.peek_next_token_or_error()?;
                    expr = Syntax::StaticAccess {
                        position: TokenPosition {
                            line: position.cursor.line,
                            column: position.cursor.column,
                        },
                        target: Box::from(expr),
                        index: {
                            let primary = self.parse_primary()?;
                            match primary {
                                Syntax::Identifier { .. } => Box::from(primary),
                                _ => return parse_error!(position.cursor, ParserErrorType::InvalidStaticAccess)
                            }
                        },
                    }
                }
                _ => break
            }
        }

        Ok(expr)
    }

    // primary
    fn parse_primary(&mut self) -> Result<Syntax, ParserError> {
        let matched = self.next_token_or_error()?;

        match matched.token {
            Token::Integer => Ok(Syntax::Integer(matched.text.parse::<i64>().unwrap())),
            Token::Float => Ok(Syntax::Float(matched.text.parse::<f64>().unwrap())),
            Token::Boolean => Ok(Syntax::Bool(matched.text.parse::<bool>().unwrap())),
            Token::Null => Ok(Syntax::Null),
            Token::New => self.parse_new_object(matched),
            Token::String => Ok(Parser::parse_string(matched)),
            Token::Identifier => Ok(Syntax::Identifier {
                position: TokenPosition {
                    line: matched.cursor.line,
                    column: matched.cursor.column,
                },
                name: matched.text,
            }),
            Token::LeftSquareBracket => self.parse_array(),
            Token::LeftCurlyBracket => self.parse_map(),
            Token::LeftParenthesis => {
                let expr = self.parse_expression()?;
                self.skip_next_token_or_error()?;
                Ok(expr)
            }

            _ => parse_error!(matched.cursor, ParserErrorType::InvalidExpressionItem(matched.text))
        }
    }

    fn parse_array(&mut self) -> Result<Syntax, ParserError> {
        let mut items = vec![];

        while self.lexer.has_more_tokens() {
            let matched = self.peek_next_token_or_error()?;

            match matched.token {
                Token::RightSquareBracket => {
                    self.skip_next_token_or_error()?;
                    break;
                }
                Token::Comma => {
                    self.skip_next_token_or_error()?;
                    continue;
                }
                _ => items.push(self.parse_expression()?),
            }
        }

        Ok(Syntax::Array(items))
    }

    fn parse_map(&mut self) -> Result<Syntax, ParserError> {
        let mut items = HashMap::new();

        while self.lexer.has_more_tokens() {
            let matched = self.peek_next_token_or_error()?;

            match matched.token {
                Token::RightCurlyBracket => {
                    self.skip_next_token_or_error()?;
                    break;
                }
                Token::Comma => {
                    self.skip_next_token_or_error()?;
                    continue;
                }
                Token::Identifier => {
                    let key = self.match_next_token_or_error(Token::Identifier)?;
                    self.skip_matched_token_or_error(Token::Colon)?;
                    let value = self.parse_expression()?;
                    items.insert(key.text, value);
                }
                _ => return parse_error!(matched.cursor, ParserErrorType::InvalidMapItem(matched.text))
            }
        }

        Ok(Syntax::Dictionary(items))
    }

    fn parse_string(matched: MatchedToken<Token>) -> Syntax {
        // remove double quotes from each end
        let mut text = matched.text.to_string();
        text.remove(0);
        text.pop();

        Syntax::String(text)
    }

    fn parse_new_object(&mut self, matched: MatchedToken<Token>) -> Result<Syntax, ParserError> {

        // get class name
        let Syntax::Call { target, args, .. } = self.parse_expression()? else {
            return parse_error!(matched.cursor, ParserErrorType::InvalidNewObject);
        };

        Ok(Syntax::NewObject {
            position: TokenPosition {
                line: matched.cursor.line,
                column: matched.cursor.column,
            },
            target: target,
            args: args,
        })
    }

    fn parse_arguments(&mut self) -> Result<Vec<Syntax>, ParserError> {
        let mut args = vec![];

        while self.lexer.has_more_tokens() {
            let matched = self.peek_next_token_or_error()?;

            match matched.token {
                Token::LeftParenthesis => {
                    self.skip_next_token_or_error()?;
                    continue;
                }
                Token::Comma => {
                    self.skip_next_token_or_error()?;
                    continue;
                }
                Token::RightParenthesis => {
                    self.skip_next_token_or_error()?;
                    break;
                }
                _ => args.push(self.parse_expression()?),
            }
        }

        Ok(args)
    }

    fn parse_array_access(&mut self) -> Result<Syntax, ParserError> {
        let mut expr = None;

        while self.lexer.has_more_tokens() {
            let matched = self.peek_next_token_or_error()?;

            match matched.token {
                Token::LeftSquareBracket => {
                    self.skip_next_token_or_error()?;
                    continue;
                }
                Token::RightSquareBracket => {
                    self.skip_next_token_or_error()?;
                    break;
                }
                _ => expr = Some(self.parse_expression()?),
            }
        }

        match expr {
            Some(expr) => Ok(expr),
            None => parse_error!(self.lexer.get_cursor(), ParserErrorType::InvalidArrayAccess)
        }
    }

    pub fn is_expression_token(token: Token) -> bool {
        match token {
            Token::Integer => true,
            Token::Float => true,
            Token::String => true,
            Token::Boolean => true,
            Token::Identifier => true,
            Token::Null => true,
            Token::LeftParenthesis => true,
            Token::Not => true,
            Token::New => true,
            _ => false
        }
    }
}


#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use super::*;

    macro_rules! run_expr_test_ok {
        ($source:expr, $expected:expr) => {
            let r = match Parser::new($source).parse_expression() {
                Ok(r) => r,
                Err(e) => {
                    assert!(false, "bad parse: {:?}", e);
                    return;
                }
            };

            assert_eq!(r, $expected);
        }
    }

    macro_rules! run_expr_test_err {
        ($source:expr, $expected:expr) => {
            match Parser::new($source).parse_expression() {
                Ok(m) => {
                    println!("{:?}", m);
                    assert!(false, "Expected error, got expression: {:?}", m);
                    return;
                }
                Err(_) => $expected
            };
        }
    }

    #[test]
    fn test_long_expression() {
        run_expr_test_ok!("1 * 2 + 3 / 4 ^ 6 == 2 + (5 * magic(x, y))", Syntax::Eq {
            expr1: Box::from(Syntax::Add {
                expr1: Box::from(Syntax::Mul {
                    expr1: Box::from(Syntax::Integer(1)),
                    expr2: Box::from(Syntax::Integer(2)),
                }),
                expr2: Box::from(Syntax::Div {
                    expr1: Box::from(Syntax::Integer(3)),
                    expr2: Box::from(Syntax::Pow {
                        expr1: Box::from(Syntax::Integer(4)),
                        expr2: Box::from(Syntax::Integer(6)),
                    }),
                }),
            }),
            expr2: Box::from(Syntax::Add {
                expr1: Box::from(Syntax::Integer(2)),
                expr2: Box::from(Syntax::Mul {
                    expr1: Box::from(Syntax::Integer(5)),
                    expr2: Box::from(Syntax::Call {
                        position: TokenPosition { line: 1, column: 31 },
                        target: Box::from(Syntax::Identifier {
                            position: TokenPosition { line: 1, column: 31 },
                            name: String::from("magic"),
                        }),
                        args: vec![
                            Syntax::Identifier {
                                position: TokenPosition { line: 1, column: 37 },
                                name: String::from("x"),
                            },
                            Syntax::Identifier {
                                position: TokenPosition { line: 1, column: 40 },
                                name: String::from("y"),
                            },
                        ],
                    }),
                }),
            }),
        });
    }

    #[test]
    fn test_bracket_math_with_multiply() {
        run_expr_test_ok!("(1 + 2) * 3", Syntax::Mul {
            expr1: Box::new(Syntax::Add {
                expr1: Box::new(Syntax::Integer(1)),
                expr2: Box::new(Syntax::Integer(2)),
            }),
            expr2: Box::new(Syntax::Integer(3)),
        });
    }

    #[test]
    fn test_double_bracket_math_with_boolean() {
        run_expr_test_ok!("((1 + 2) == 3) == true", Syntax::Eq {
            expr1: Box::new(Syntax::Eq {
                expr1: Box::new(Syntax::Add {
                    expr1: Box::new(Syntax::Integer(1)),
                    expr2: Box::new(Syntax::Integer(2)),
                }),
                expr2: Box::new(Syntax::Integer(3)),
            }),
            expr2: Box::new(Syntax::Bool(true)),
        });
    }

    #[test]
    fn test_order_of_operation_plus_and_multiply() {
        run_expr_test_ok!("9 + 7 * 8", Syntax::Add {
            expr1: Box::from(Syntax::Integer(9)),
            expr2: Box::from(Syntax::Mul {
                expr1: Box::from(Syntax::Integer(7)),
                expr2: Box::from(Syntax::Integer(8)),
            }),
        });
    }

    #[test]
    fn test_with_identifier() {
        run_expr_test_ok!("9 + a * 8", Syntax::Add {
            expr1: Box::from(Syntax::Integer(9)),
            expr2: Box::from(Syntax::Mul {
                expr1: Box::from(Syntax::Identifier {
                    position: TokenPosition { line: 1, column: 5 },
                    name: String::from("a"),
                }),
                expr2: Box::from(Syntax::Integer(8)),
            }),
        });
    }

    #[test]
    fn test_with_function_call() {
        run_expr_test_ok!("a() + 7", Syntax::Add {
            expr1: Box::from(Syntax::Call {
                position: TokenPosition { line: 1, column: 1 },
                target: Box::from(Syntax::Identifier {
                    position: TokenPosition { line: 1, column: 1 },
                    name: String::from("a"),
                }),
                args: vec![],
            }),
            expr2: Box::from(Syntax::Integer(7)),
        });
    }

    #[test]
    fn test_add() {
        run_expr_test_ok!("1 + 2", Syntax::Add {
            expr1: Box::from(Syntax::Integer(1)),
            expr2: Box::from(Syntax::Integer(2)),
        });
    }

    #[test]
    fn test_subtract() {
        run_expr_test_ok!("1 - 2", Syntax::Sub {
            expr1: Box::from(Syntax::Integer(1)),
            expr2: Box::from(Syntax::Integer(2)),
        });
    }

    #[test]
    fn test_multiply() {
        run_expr_test_ok!("1 * 2", Syntax::Mul {
            expr1: Box::from(Syntax::Integer(1)),
            expr2: Box::from(Syntax::Integer(2)),
        });
    }

    #[test]
    fn test_divide() {
        run_expr_test_ok!("1 / 2", Syntax::Div {
            expr1: Box::from(Syntax::Integer(1)),
            expr2: Box::from(Syntax::Integer(2)),
        });
    }

    // --- Comparison operators ---

    #[test]
    fn test_equals() {
        run_expr_test_ok!("1 == 2", Syntax::Eq {
            expr1: Box::from(Syntax::Integer(1)),
            expr2: Box::from(Syntax::Integer(2)),
        });
    }

    #[test]
    fn test_not_equals() {
        run_expr_test_ok!("1 != 2", Syntax::Ne {
            expr1: Box::from(Syntax::Integer(1)),
            expr2: Box::from(Syntax::Integer(2)),
        });
    }

    #[test]
    fn test_not_equals_with_identifier() {
        run_expr_test_ok!("a != 1", Syntax::Ne {
            expr1: Box::from(Syntax::Identifier {
                position: TokenPosition { line: 1, column: 1 },
                name: String::from("a"),
            }),
            expr2: Box::from(Syntax::Integer(1)),
        });
    }

    #[test]
    fn test_less_than() {
        run_expr_test_ok!("1 < 2", Syntax::Lt {
            expr1: Box::from(Syntax::Integer(1)),
            expr2: Box::from(Syntax::Integer(2)),
        });
    }

    #[test]
    fn test_less_than_or_equal() {
        run_expr_test_ok!("1 <= 2", Syntax::Le {
            expr1: Box::from(Syntax::Integer(1)),
            expr2: Box::from(Syntax::Integer(2)),
        });
    }

    #[test]
    fn test_greater_than() {
        run_expr_test_ok!("1 > 2", Syntax::Gt {
            expr1: Box::from(Syntax::Integer(1)),
            expr2: Box::from(Syntax::Integer(2)),
        });
    }

    #[test]
    fn test_greater_than_or_equal() {
        run_expr_test_ok!("1 >= 2", Syntax::Ge {
            expr1: Box::from(Syntax::Integer(1)),
            expr2: Box::from(Syntax::Integer(2)),
        });
    }

    #[test]
    fn test_pow() {
        run_expr_test_ok!("1 ^ 3", Syntax::Pow {
            expr1: Box::from(Syntax::Integer(1)),
            expr2: Box::from(Syntax::Integer(3)),
        });
    }

    // --- Literals ---

    #[test]
    fn test_integer() {
        run_expr_test_ok!("1", Syntax::Integer(1));
    }

    #[test]
    fn test_float() {
        run_expr_test_ok!("1.5", Syntax::Float(1.5));
    }

    #[test]
    fn test_string() {
        run_expr_test_ok!("\"hello\"", Syntax::String(String::from("hello")));
    }

    #[test]
    fn test_bool_true() {
        run_expr_test_ok!("true", Syntax::Bool(true));
    }

    #[test]
    fn test_bool_false() {
        run_expr_test_ok!("false", Syntax::Bool(false));
    }

    // -- Function calls ---

    #[test]
    fn test_function_call_with_no_args() {
        run_expr_test_ok!("a()", Syntax::Call {
            position: TokenPosition { line: 1, column: 1 },
            target: Box::from(Syntax::Identifier {
                position: TokenPosition { line: 1, column: 1 },
                name: String::from("a"),
            }),
            args: vec![],
        });
    }

    #[test]
    fn test_function_call_with_one_arg() {
        run_expr_test_ok!("a(1)", Syntax::Call {
            position: TokenPosition { line: 1, column: 1 },
            target: Box::from(Syntax::Identifier {
                position: TokenPosition { line: 1, column: 1 },
                name: String::from("a"),
            }),
            args: vec![Syntax::Integer(1)],
        });
    }

    #[test]
    fn test_function_call_with_two_args() {
        run_expr_test_ok!("a(1, 2)", Syntax::Call {
            position: TokenPosition { line: 1, column: 1 },
            target: Box::from(Syntax::Identifier {
                position: TokenPosition { line: 1, column: 1 },
                name: String::from("a"),
            }),
            args: vec![Syntax::Integer(1), Syntax::Integer(2)],
        });
    }

    // --- Logical operators ---

    #[test]
    fn test_logical_and() {
        run_expr_test_ok!("a == 1 and b == 2", Syntax::And {
            expr1: Box::from(Syntax::Eq {
                expr1: Box::from(Syntax::Identifier {
                    position: TokenPosition { line: 1, column: 1 },
                    name: String::from("a"),
                }),
                expr2: Box::from(Syntax::Integer(1)),
            }),
            expr2: Box::from(Syntax::Eq {
                expr1: Box::from(Syntax::Identifier {
                    position: TokenPosition { line: 1, column: 12 },
                    name: String::from("b"),
                }),
                expr2: Box::from(Syntax::Integer(2)),
            }),
        });
    }

    #[test]
    fn test_logical_or() {
        run_expr_test_ok!("a == 9.9 or b == 2", Syntax::Or {
            expr1: Box::from(Syntax::Eq {
                expr1: Box::from(Syntax::Identifier {
                    position: TokenPosition { line: 1, column: 1 },
                    name: String::from("a"),
                }),
                expr2: Box::from(Syntax::Float(9.9)),
            }),
            expr2: Box::from(Syntax::Eq {
                expr1: Box::from(Syntax::Identifier {
                    position: TokenPosition { line: 1, column: 13 },
                    name: String::from("b"),
                }),
                expr2: Box::from(Syntax::Integer(2)),
            }),
        });
    }

    #[test]
    fn test_logical_not() {
        run_expr_test_ok!("not a == 1", Syntax::Not {
            expr: Box::from(Syntax::Eq {
                expr1: Box::from(Syntax::Identifier {
                    position: TokenPosition { line: 1, column: 5 },
                    name: String::from("a"),
                }),
                expr2: Box::from(Syntax::Integer(1)),
            }),
        });
    }

    #[test]
    fn test_order_of_and_or() {
        run_expr_test_ok!("x > 0 AND y < 10 OR z == 5", Syntax::Or {
            expr1: Box::from(Syntax::And {
                expr1: Box::from(Syntax::Gt {
                    expr1: Box::from(Syntax::Identifier {
                        position: TokenPosition { line: 1, column: 1 },
                        name: String::from("x"),
                    }),
                    expr2: Box::from(Syntax::Integer(0)),
                }),
                expr2: Box::from(Syntax::Lt {
                    expr1: Box::from(Syntax::Identifier {
                        position: TokenPosition { line: 1, column: 11 },
                        name: String::from("y"),
                    }),
                    expr2: Box::from(Syntax::Integer(10)),
                }),
            }),
            expr2: Box::from(Syntax::Eq {
                expr1: Box::from(Syntax::Identifier {
                    position: TokenPosition { line: 1, column: 21 },
                    name: String::from("z"),
                }),
                expr2: Box::from(Syntax::Integer(5)),
            }),
        });
    }

    // --- Data access ---

    #[test]
    fn test_array_access() {
        run_expr_test_ok!("a[1]", Syntax::ArrayAccess {
            position: TokenPosition { line: 1, column: 3 },
            target: Box::from(Syntax::Identifier {
                position: TokenPosition { line: 1, column: 1 },
                name: String::from("a"),
            }),
            index: Box::from(Syntax::Integer(1)),
        });
    }

    #[test]
    fn test_array_access_with_math() {
        run_expr_test_ok!("a[1] + b[2]", Syntax::Add {
            expr1: Box::from(Syntax::ArrayAccess {
                position: TokenPosition { line: 1, column: 3 },
                target: Box::from(Syntax::Identifier {
                    position: TokenPosition { line: 1, column: 1 },
                    name: String::from("a"),
                }),
                index: Box::from(Syntax::Integer(1)),
            }),
            expr2: Box::from(Syntax::ArrayAccess {
                position: TokenPosition { line: 1, column: 10 },
                target: Box::from(Syntax::Identifier {
                    position: TokenPosition { line: 1, column: 8 },
                    name: String::from("b"),
                }),
                index: Box::from(Syntax::Integer(2)),
            }),
        });
    }

    #[test]
    fn test_multi_dimension_array_access() {
        run_expr_test_ok!("a[1][2]", Syntax::ArrayAccess {
            position: TokenPosition { line: 1, column: 6 },
            target: Box::from(Syntax::ArrayAccess {
                position: TokenPosition { line: 1, column: 3 },
                target: Box::from(Syntax::Identifier {
                    position: TokenPosition { line: 1, column: 1 },
                    name: String::from("a"),
                }),
                index: Box::from(Syntax::Integer(1)),
            }),
            index: Box::from(Syntax::Integer(2)),
        });
    }

    #[test]
    fn test_multi_dimension_array_access_with_function_call() {
        run_expr_test_ok!("f()[1][2]", Syntax::ArrayAccess {
            position: TokenPosition { line: 1, column: 8 },
            target: Box::from(Syntax::ArrayAccess {
                position: TokenPosition { line: 1, column: 5 },
                target: Box::from(Syntax::Call {
                    position: TokenPosition { line: 1, column: 1 },
                    target: Box::from(Syntax::Identifier {
                        position: TokenPosition { line: 1, column: 1 },
                        name: String::from("f"),
                    }),
                    args: vec![],
                }),
                index: Box::from(Syntax::Integer(1)),
            }),
            index: Box::from(Syntax::Integer(2)),
        });
    }

    #[test]
    fn test_multi_dimension_array_access_with_function_call2() {
        run_expr_test_ok!("f[1]()[2]", Syntax::ArrayAccess {
            position: TokenPosition { line: 1, column: 8 },
            target: Box::from(Syntax::Call {
                position: TokenPosition { line: 1, column: 3 },
                target: Box::from(Syntax::ArrayAccess {
                    position: TokenPosition { line: 1, column: 3 },
                    target: Box::from(Syntax::Identifier {
                        position: TokenPosition { line: 1, column: 1 },
                        name: String::from("f"),
                    }),
                    index: Box::from(Syntax::Integer(1)),
                }),
                args: vec![],
            }),
            index: Box::from(Syntax::Integer(2)),
        });
    }

    #[test]
    fn test_identifier_chain_simple() {
        run_expr_test_ok!("library.shelves.books", Syntax::MemberAccess {
            position: TokenPosition { line: 1, column: 17 },
            index: Box::from(Syntax::Identifier {
                position: TokenPosition { line: 1, column: 17 },
                name: String::from("books"),
            }),
            target: Box::from(Syntax::MemberAccess {
                position: TokenPosition { line: 1, column: 9 },
                index: Box::from(Syntax::Identifier {
                    position: TokenPosition { line: 1, column: 9 },
                    name: String::from("shelves"),
                }),
                target: Box::from(Syntax::Identifier {
                    position: TokenPosition { line: 1, column: 1 },
                    name: String::from("library"),
                }),
            }),
        });
    }

    #[test]
    fn test_identifier_chain_with_array_int_access() {
        run_expr_test_ok!("library.shelf[1].books", Syntax::MemberAccess {
            position: TokenPosition { line: 1, column: 18 },
            index: Box::from(Syntax::Identifier {
                position: TokenPosition { line: 1, column: 18 },
                name: String::from("books"),
            }),
            target: Box::from(Syntax::ArrayAccess {
                position: TokenPosition { line: 1, column: 15 },
                index: Box::from(Syntax::Integer(1)),
                target: Box::from(Syntax::MemberAccess {
                    position: TokenPosition { line: 1, column: 9 },
                    index: Box::from(Syntax::Identifier {
                        position: TokenPosition { line: 1, column: 9 },
                        name: String::from("shelf"),
                    }),
                    target: Box::from(Syntax::Identifier {
                        position: TokenPosition { line: 1, column: 1 },
                        name: String::from("library"),
                    }),
                }),
            }),
        });
    }


    #[test]
    fn test_member_call_on_function_call() {
        run_expr_test_ok!(r#"get_books().first()"#, Syntax::Call {
            position: TokenPosition { line: 1, column: 13 },
            target: Box::from(Syntax::MemberAccess {
                position: TokenPosition { line: 1, column: 13 },
                index: Box::from(Syntax::Identifier {
                    position: TokenPosition { line: 1, column: 13 },
                    name: String::from("first"),
                }),
                target: Box::from(Syntax::Call {
                    position: TokenPosition { line: 1, column: 1 },
                    target: Box::from(Syntax::Identifier {
                        position: TokenPosition { line: 1, column: 1 },
                        name: String::from("get_books"),
                    }),
                    args: vec![],
                }),
            }),
            args: vec![],
        });
    }

    #[test]
    fn test_function_call_on_parent() {
        run_expr_test_ok!(r#"svc.something()"#, Syntax::Call {
            position: TokenPosition { line: 1, column: 5 },
            target: Box::from(Syntax::MemberAccess {
                position: TokenPosition { line: 1, column: 5 },
                index: Box::from(Syntax::Identifier {
                    position: TokenPosition { line: 1, column: 5 },
                    name: String::from("something"),
                }),
                target: Box::from(Syntax::Identifier {
                    position: TokenPosition { line: 1, column: 1 },
                    name: String::from("svc"),
                }),
            }),
            args: vec![],
        });
    }

    #[test]
    fn test_enum_access() {
        run_expr_test_ok!("Color.Red", Syntax::MemberAccess {
            position: TokenPosition { line: 1, column: 7 },
            index: Box::from(Syntax::Identifier {
                position: TokenPosition { line: 1, column: 7 },
                name: String::from("Red"),
            }),
            target: Box::from(Syntax::Identifier {
                position: TokenPosition { line: 1, column: 1 },
                name: String::from("Color"),
            }),
        });
    }

    #[test]
    fn test_invalid_member_access_with_non_identifier() {
        run_expr_test_err!("books.6[a]", ParserError {
            position: TokenPosition { line: 1, column: 7 },
            error: ParserErrorType::InvalidMemberAccess,
        });
    }

    #[test]
    fn test_access_index_is_expression() {
        run_expr_test_ok!("page[chapter + 3]", Syntax::ArrayAccess {
            position: TokenPosition { line: 1, column: 6 },
            target: Box::from(Syntax::Identifier {
                position: TokenPosition { line: 1, column: 1 },
                name: String::from("page"),
            }),
            index: Box::from(Syntax::Add {
                expr1: Box::from(Syntax::Identifier {
                    position: TokenPosition { line: 1, column: 6 },
                    name: String::from("chapter"),
                }),
                expr2: Box::from(Syntax::Integer(3)),
            }),
        });
    }

    #[test]
    fn array_access_on_class_attribute() {
        run_expr_test_ok!("book.authors[0]", Syntax::ArrayAccess {
            position: TokenPosition { line: 1, column: 14 },
            target: Box::from(Syntax::MemberAccess {
                position: TokenPosition { line: 1, column: 6 },
                index: Box::from(Syntax::Identifier {
                    position: TokenPosition { line: 1, column: 6 },
                    name: String::from("authors"),
                }),
                target: Box::from(Syntax::Identifier {
                    position: TokenPosition { line: 1, column: 1 },
                    name: String::from("book"),
                }),
            }),
            index: Box::from(Syntax::Integer(0)),
        });
    }

    #[test]
    fn test_member_access_with_square_brackets
    () {
        run_expr_test_ok!(r#"library["shelves"]["books"]"#, Syntax::ArrayAccess {
            position: TokenPosition { line: 1, column: 20 },
            index: Box::from(Syntax::String(String::from("books"))),
            target: Box::from(Syntax::ArrayAccess {
                position: TokenPosition { line: 1, column: 9 },
                index: Box::from(Syntax::String(String::from("shelves"))),
                target: Box::from(Syntax::Identifier {
                    position: TokenPosition { line: 1, column: 1 },
                    name: String::from("library"),
                }),
            }),
        });
    }

    // --- New object ---

    #[test]
    fn test_new_object() {
        run_expr_test_ok!(r#"new Book("Frankenstein")"#, Syntax::NewObject {
            position: TokenPosition { line: 1, column: 1 },
            target: Box::from(Syntax::Identifier {
                position: TokenPosition { line: 1, column: 5 },
                name: String::from("Book"),
            }),
            args: vec![
                Syntax::String(String::from("Frankenstein")),
            ],
        });
    }

    #[test]
    fn new_object_from_module() {
        run_expr_test_ok!(r#"new Library::Book("Frankenstein")"#, Syntax::NewObject {
            position: TokenPosition { line: 1, column: 1 },
            target: Box::from(Syntax::StaticAccess {
                position: TokenPosition { line: 1, column: 14 },
                index: Box::from(Syntax::Identifier {
                    position: TokenPosition { line: 1, column: 14 },
                    name: String::from("Book"),
                }),
                target: Box::from(Syntax::Identifier {
                    position: TokenPosition { line: 1, column: 5 },
                    name: String::from("Library"),
                }),
            }),
            args: vec![
                Syntax::String(String::from("Frankenstein")),
            ],
        });
    }

    #[test]
    fn test_member_call_on_new_object() {
        run_expr_test_ok!(r#"new Book("Christmas Carol").publish()"#, Syntax::Call {
            position: TokenPosition { line: 1, column: 29 },
            target: Box::from(Syntax::MemberAccess {
                position: TokenPosition { line: 1, column: 29 },
                index: Box::from(Syntax::Identifier {
                    position: TokenPosition { line: 1, column: 29 },
                    name: String::from("publish"),
                }),
                target: Box::from(Syntax::NewObject {
                    position: TokenPosition { line: 1, column: 1 },
                    target: Box::from(Syntax::Identifier {
                        position: TokenPosition { line: 1, column: 5 },
                        name: String::from("Book"),
                    }),
                    args: vec![
                        Syntax::String(String::from("Christmas Carol")),
                    ],
                }),
            }),
            args: vec![],
        });
    }

    // --- Array ---

    #[test]
    fn test_array() {
        run_expr_test_ok!("[1, 2, 3]", Syntax::Array(vec![
            Syntax::Integer(1),
            Syntax::Integer(2),
            Syntax::Integer(3),
        ]));
    }

    #[test]
    fn test_array_with_math() {
        run_expr_test_ok!("[1, (a + b), 3]", Syntax::Array(vec![
            Syntax::Integer(1),
            Syntax::Add {
                expr1: Box::from(Syntax::Identifier {
                    position: TokenPosition { line: 1, column: 6 },
                    name: String::from("a"),
                }),
                expr2: Box::from(Syntax::Identifier {
                    position: TokenPosition { line: 1, column: 10 },
                    name: String::from("b"),
                }),
            },
            Syntax::Integer(3),
        ]));
    }

    #[test]
    fn test_deep_array() {
        run_expr_test_ok!("[1, 2, [3, 4, [5, 6]], [7, 8]]", Syntax::Array(vec![
            Syntax::Integer(1),
            Syntax::Integer(2),
            Syntax::Array(vec![
                Syntax::Integer(3),
                Syntax::Integer(4),
                Syntax::Array(vec![
                    Syntax::Integer(5),
                    Syntax::Integer(6),
                ]),
            ]),
            Syntax::Array(vec![
                Syntax::Integer(7),
                Syntax::Integer(8),
            ]),
        ]));
    }

    // --- Maps ---

    #[test]
    fn map_with_1_dimension() {
        let mut expected_map = HashMap::new();
        expected_map.insert(String::from("one"), Syntax::Integer(1));
        expected_map.insert(String::from("two"), Syntax::Integer(2));
        expected_map.insert(String::from("three"), Syntax::Integer(3));

        run_expr_test_ok!(r#"{one: 1, two : 2, three : 3}"#, Syntax::Dictionary(expected_map));
    }

    #[test]
    fn map_with_2_dimensions() {
        let mut expected_map2 = HashMap::new();
        expected_map2.insert(String::from("five"), Syntax::Integer(5));
        expected_map2.insert(String::from("six"), Syntax::Integer(6));

        let mut expected_map3 = HashMap::new();
        expected_map3.insert(String::from("eight"), Syntax::Integer(8));
        expected_map3.insert(String::from("nine"), Syntax::Integer(9));

        let mut expected_map = HashMap::new();
        expected_map.insert(String::from("one"), Syntax::Integer(1));
        expected_map.insert(String::from("two"), Syntax::Integer(2));
        expected_map.insert(String::from("three"), Syntax::Integer(3));
        expected_map.insert(String::from("four"), Syntax::Dictionary(expected_map2));
        expected_map.insert(String::from("seven"), Syntax::Dictionary(expected_map3));

        run_expr_test_ok!(r#"{one: 1, two : 2, three : 3, four: {five: 5, six: 6}, seven: {eight: 8, nine: 9}}"#, Syntax::Dictionary(expected_map));
    }
}