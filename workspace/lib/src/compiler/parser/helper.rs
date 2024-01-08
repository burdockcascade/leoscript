use crate::compiler::error::{ParserError, ParserErrorType};
use crate::compiler::parser::lexer::lexer::MatchedToken;
use crate::compiler::parser::lexer::Token;
use crate::compiler::parser::Parser;
use crate::compiler::parser::TokenPosition;
use crate::parser_error;

impl Parser {
    pub fn skip_matched_token_or_error(&mut self, expected: Token) -> Result<(), ParserError> {
        match self.match_next_token_or_error(expected) {
            Ok(_) => Ok(()),
            Err(e) => Err(e)
        }
    }

    pub fn match_next_token_or_error(&mut self, expected: Token) -> Result<MatchedToken<Token>, ParserError> {
        match self.next_token_or_error() {
            Ok(m) if m.token == expected => Ok(m),
            Ok(m) => return parser_error!(m.cursor, ParserErrorType::UnexpectedToken { expected, found: m.token }),
            Err(e) => Err(e)
        }
    }

    pub fn next_token_or_error(&mut self) -> Result<MatchedToken<Token>, ParserError> {
        match self.lexer.next() {
            Some(Ok(m)) => Ok(m),
            Some(Err(e)) => return parser_error!(self.lexer.get_cursor(), ParserErrorType::UnexpectedParserError(e)),
            None => parser_error!(self.lexer.get_cursor(), ParserErrorType::NoMatchFound)
        }
    }

    pub fn skip_next_token_or_error(&mut self) -> Result<(), ParserError> {
        match self.lexer.skip() {
            Ok(_) => Ok(()),
            Err(e) => parser_error!(self.lexer.get_cursor(), ParserErrorType::UnexpectedParserError(e)),
        }
    }

    pub fn peek_next_token_or_error(&mut self) -> Result<MatchedToken<Token>, ParserError> {
        match self.lexer.peek() {
            Some(Ok(m)) => Ok(m),
            Some(Err(e)) => return parser_error!(self.lexer.get_cursor(), ParserErrorType::UnexpectedParserError(e)),
            None => parser_error!(self.lexer.get_cursor(), ParserErrorType::NoMatchFound)
        }
    }

    pub fn peek_next_token_match(&mut self, expected: Token) -> bool {
        match self.lexer.peek() {
            Some(Ok(m)) => m.token == expected,
            _ => false
        }
    }
}