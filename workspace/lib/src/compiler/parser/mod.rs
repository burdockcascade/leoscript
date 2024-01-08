use crate::compiler::codegen::syntax::Syntax;
use crate::compiler::codegen::syntax::TokenPosition;
use crate::compiler::error::{ParserError, ParserErrorType};
use crate::compiler::parser::lexer::{get_lexer, Token};
use crate::compiler::parser::lexer::lexer::Lexer;
use crate::parse_error;

pub mod toplevel;
mod function;
mod variable;
mod expression;
mod looping;
mod helper;
mod branching;
mod identifier;
pub mod lexer;
mod import;

#[derive(Debug)]
pub struct ParserResult {
    pub syntax_tree: Vec<Syntax>,
    pub parser_time: std::time::Duration,
}

pub struct Parser {
    syntax_tree: Vec<Syntax>,
    lexer: Lexer<Token>,
}

impl Default for Parser {
    fn default() -> Self {
        Parser {
            syntax_tree: vec![],
            lexer: Default::default(),
        }
    }
}

impl Parser {
    pub fn new(src: &str) -> Self {
        Parser {
            syntax_tree: vec![],
            lexer: get_lexer(src),
        }
    }

    pub fn parse(source: &str) -> Result<ParserResult, ParserError> {
        let parser_timer = std::time::Instant::now();

        let mut p = Parser::new(source);

        p.parse_script()?;

        Ok(ParserResult {
            syntax_tree: p.syntax_tree,
            parser_time: parser_timer.elapsed(),
        })
    }
}
