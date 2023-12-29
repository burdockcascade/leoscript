use crate::compiler::codegen::syntax::Syntax;

pub mod script;

#[derive(Debug)]
pub struct ParserResult {
    pub tokens: Vec<Syntax>,
    pub parser_time: std::time::Duration,
}

#[macro_export]
macro_rules! next_token {
    ($lexer:expr) => {
        match $lexer.next() {
            Some(Ok(m)) => {
                m
            },
            _ => {
                return Err(ParserError {
                    error: ParserErrorType::UnexpectedError,
                    position: TokenPosition {
                        line: $lexer.get_cursor().line,
                        column: $lexer.get_cursor().column,
                    },
                })
            }
        }
    };
}

#[macro_export]
macro_rules! peek_next_token {
    ($lexer:expr) => {
        match $lexer.peek() {
            Some(Ok(m)) => m,
            _ => {
                return Err(ParserError {
                    error: ParserErrorType::UnexpectedError,
                    position: TokenPosition {
                        line: $lexer.get_cursor().line,
                        column: $lexer.get_cursor().column,
                    },
                })
            }
        }
    };
}