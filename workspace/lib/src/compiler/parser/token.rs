use std::collections::HashMap;
use std::fmt::Display;

use crate::compiler::parser::Span;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Import {
        position: TokenPosition,
        source: Vec<Token>,
    },

    Comment {
        position: TokenPosition,
        text: String,
    },
    Print {
        position: TokenPosition,
        expr: Box<Token>,
    },

    Sleep {
        position: TokenPosition,
        expr: Box<Token>,
    },

    Constructor {
        position: TokenPosition,
        input: Vec<Token>,
        body: Vec<Token>,
    },

    Function {
        position: TokenPosition,
        function_name: Box<Token>,
        is_static: bool,
        scope: Option<Box<Token>>,
        return_type: Option<Box<Token>>,
        input: Vec<Token>,
        body: Vec<Token>,
    },

    AnonFunction {
        position: TokenPosition,
        input: Vec<Token>,
        body: Vec<Token>,
    },

    Module {
        position: TokenPosition,
        module_name: Box<Token>,
        body: Vec<Token>,
    },

    Class {
        position: TokenPosition,
        class_name: Box<Token>,
        body: Vec<Token>,
    },

    Identifier {
        position: TokenPosition,
        name: String,
    },

    DotChain {
        position: TokenPosition,
        start: Box<Token>,
        chain: Vec<Token>,
    },

    Variable {
        position: TokenPosition,
        name: String,
        as_type: Option<String>,
        value: Option<Box<Token>>,
    },

    Attribute {
        position: TokenPosition,
        name: String,
        as_type: Option<String>,
        value: Option<Box<Token>>,
    },

    Constant {
        position: TokenPosition,
        name: String,
        value: Box<Token>,
    },

    NewObject {
        position: TokenPosition,
        name: Box<Token>,
        input: Vec<Token>,
    },

    Assign {
        position: TokenPosition,
        ident: Box<Token>,
        value: Box<Token>,
    },

    Null,
    Integer(i64),
    Float(f64),
    Bool(bool),
    String(String),

    Array(Vec<Token>),
    Dictionary(HashMap<String, Token>),

    CollectionIndex(Box<Token>),

    Enum {
        position: TokenPosition,
        name: String,
        items: Vec<Token>,
    },

    Not { expr: Box<Token> },
    And { expr1: Box<Token>, expr2: Box<Token> },
    Or { expr1: Box<Token>, expr2: Box<Token> },
    Eq { expr1: Box<Token>, expr2: Box<Token> },
    Ne { expr1: Box<Token>, expr2: Box<Token> },
    Lt { expr1: Box<Token>, expr2: Box<Token> },
    Le { expr1: Box<Token>, expr2: Box<Token> },
    Gt { expr1: Box<Token>, expr2: Box<Token> },
    Ge { expr1: Box<Token>, expr2: Box<Token> },
    Add { expr1: Box<Token>, expr2: Box<Token> },
    Sub { expr1: Box<Token>, expr2: Box<Token> },
    Mul { expr1: Box<Token>, expr2: Box<Token> },
    Div { expr1: Box<Token>, expr2: Box<Token> },
    Pow { expr1: Box<Token>, expr2: Box<Token> },

    IfChain {
        position: TokenPosition,
        chain: Vec<Token>,
    },
    If {
        position: TokenPosition,
        condition: Box<Token>,
        body: Vec<Token>,
    },
    Else {
        position: TokenPosition,
        body: Vec<Token>,
    },

    Match {
        position: TokenPosition,
        expr: Box<Token>,
        arms: Vec<Token>,
        default: Option<Box<Token>>,
    },

    Case {
        position: TokenPosition,
        condition: Box<Token>,
        body: Vec<Token>,
    },

    DefaultCase {
        position: TokenPosition,
        body: Vec<Token>,
    },

    WhileLoop {
        position: TokenPosition,
        condition: Box<Token>,
        body: Vec<Token>,
    },

    ForEach {
        position: TokenPosition,
        ident: Box<Token>,
        collection: Box<Token>,
        body: Vec<Token>,
    },

    ForI {
        position: TokenPosition,
        ident: Box<Token>,
        start: Box<Token>,
        step: Box<Token>,
        end: Box<Token>,
        body: Vec<Token>,
    },

    Break {
        position: TokenPosition,
    },
    Continue {
        position: TokenPosition,
    },

    Call {
        position: TokenPosition,
        name: Box<Token>,
        input: Vec<Token>,
    },

    Return {
        position: TokenPosition,
        expr: Option<Box<Token>>,
    },
}

pub enum Visibility {
    Public,
    Private,
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Token::Identifier { name, .. } => name.to_string(),
            Token::String(s) => s.to_string(),
            _ => unimplemented!("Token::to_string() not implemented for {:?}", self)
        };
        write!(f, "{}", str)
    }
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub struct TokenPosition {
    pub line: usize,
    pub column: usize,
}

impl Default for TokenPosition {
    fn default() -> Self {
        TokenPosition {
            line: 0,
            column: 0,
        }
    }
}

impl TokenPosition {
    pub fn new(src: &Span) -> Self {
        TokenPosition {
            line: src.location_line() as usize,
            column: src.get_column(),
        }
    }
}

