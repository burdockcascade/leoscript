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

        while p.lexer.has_more_tokens() {
            let matched = p.peek_next_token_or_error()?;

            let syntax = match matched.token {
                Token::Class => p.parse_class()?,
                Token::Module => p.parse_module()?,
                Token::Function => p.parse_function()?,
                Token::Enum => p.parse_enum()?,
                Token::Import => p.parse_import()?,
                _ => return parse_error!(matched.cursor, ParserErrorType::UnwantedToken(matched.token))
            };

            p.syntax_tree.push(syntax);
        }

        Ok(ParserResult {
            syntax_tree: p.syntax_tree,
            parser_time: parser_timer.elapsed(),
        })
    }
}
