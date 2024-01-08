use lexer::Matcher;

use crate::{ignore_regex, match_regex, match_token};
use crate::compiler::parser::lexer::lexer::Lexer;

pub mod lexer;

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Function,
    Enum,
    Import,
    Class,
    Constructor,
    Module,
    Return,
    End,

    Var,
    Constant,
    As,
    Attribute,

    // Loops
    While,
    For,
    To,
    In,
    Step,
    Do,
    Continue,
    Break,

    // If
    If,
    Then,
    Else,
    Match,
    Case,
    Default,

    // Comments
    Comment,

    // Literals
    Integer,
    Float,
    String,
    Boolean,
    Identifier,
    Null,

    // Symbols
    LeftParenthesis,
    RightParenthesis,
    RightSquareBracket,
    LeftSquareBracket,
    RightCurlyBracket,
    LeftCurlyBracket,
    Comma,
    SingleEquals,
    Semicolon,
    Dot,
    Colon,
    DoubleColon,

    // Operators
    DoubleEquals,
    Plus,
    Minus,
    Mul,
    Div,
    Pow,
    Not,
    And,
    Or,
    NotEquals,

    // Comparison operators
    Lt,
    Le,
    Gt,
    Ge,

}

pub fn get_lexer(input: &str) -> Lexer<Token> {
    let tokens = vec![

        // Whitespace
        ignore_regex!(r"^\s+"),

        // Comments
        ignore_regex!(r"^--[^\n]*"),

        // Keywords
        match_token!("function", Token::Function),
        match_token!("constructor", Token::Constructor),
        match_token!("enum", Token::Enum),
        match_token!("class", Token::Class),
        match_token!("module", Token::Module),
        match_token!("import", Token::Import),
        match_token!("return", Token::Return),
        match_token!("end", Token::End),

        // Types
        match_token!("var ", Token::Var),
        match_token!("const ", Token::Constant),
        match_token!("as ", Token::As),
        match_token!("attribute ", Token::Attribute),

        // Symbols
        match_token!("(", Token::LeftParenthesis),
        match_token!(")", Token::RightParenthesis),
        match_token!("[", Token::LeftSquareBracket),
        match_token!("]", Token::RightSquareBracket),
        match_token!("{", Token::LeftCurlyBracket),
        match_token!("}", Token::RightCurlyBracket),
        match_token!(",", Token::Comma),
        match_token!(".", Token::Dot),
        match_token!("::", Token::DoubleColon),
        match_token!(":", Token::Colon),

        // Literals
        match_regex!(r"^[0-9]+\.[0-9]+", Token::Float),
        match_regex!(r"^[-]?[0-9]+", Token::Integer),
        match_regex!(r#"^"[^"]*""#, Token::String),
        match_token!("true", Token::Boolean),
        match_token!("false", Token::Boolean),
        match_token!("null", Token::Null),

        // If
        match_token!("if", Token::If),
        match_token!("then", Token::Then),
        match_token!("else", Token::Else),
        match_token!("match", Token::Match),
        match_token!("case", Token::Case),
        match_token!("default", Token::Default),

        // Loops
        // fixme handle incorrect token matches better
        match_token!("while ", Token::While),
        match_token!("for ", Token::For),
        match_token!("to ", Token::To),
        match_token!("in ", Token::In),
        match_token!("step ", Token::Step),
        match_regex!(r"^\bdo\s+", Token::Do),
        match_token!("continue", Token::Continue),
        match_token!("break", Token::Break),

        // Operators
        match_token!("==", Token::DoubleEquals),
        match_token!("!=", Token::NotEquals),
        match_token!("+", Token::Plus),
        match_token!("-", Token::Minus),
        match_token!("*", Token::Mul),
        match_token!("/", Token::Div),
        match_token!("^", Token::Pow),

        // Comparison operators
        match_token!("<=", Token::Le),
        match_token!(">=", Token::Ge),
        match_token!("<", Token::Lt),
        match_token!(">", Token::Gt),

        // Logical operators
        match_token!("not", Token::Not),
        match_token!("and ", Token::And),
        match_token!("or ", Token::Or),
        match_token!("=", Token::SingleEquals),

        // always last because its greedy
        match_regex!("^[a-zA-Z0-9_]+", Token::Identifier),
    ];

    Lexer::new(input, tokens)
}
