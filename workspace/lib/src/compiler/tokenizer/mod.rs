use lexer::Matcher;
use crate::compiler::tokenizer::lexer::{Tokenizer, TokenOptions};

pub mod lexer;

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Function,
    End,
    Comment,
    NoArgs,
    Identifier,
}

pub fn get_tokenizer(input: &str) -> Tokenizer<Token> {

    let tokens = vec![
        Matcher::Token { value: Token::Function, condition: String::from("function"), case_sensitive: true },
        Matcher::Token { value: Token::End, condition: String::from("end"), case_sensitive: false },
        Matcher::Regex { value: Token::Comment, condition: String::from(r"^--[^\n]*") },
        Matcher::Regex { value: Token::NoArgs, condition: String::from(r#"^\([\s]*\)"#) },
        Matcher::Regex { value: Token::Identifier, condition: String::from(r"^[a-zA-Z_][a-zA-Z0-9_]*") },
    ];

    Tokenizer::new(input, tokens, TokenOptions {
        ignore_whitespace: true,
    })
}
