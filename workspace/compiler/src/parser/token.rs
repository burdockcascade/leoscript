use std::collections::HashMap;
use std::fmt::Display;
use crate::combinator::ParserPosition;


#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Import {
        position: ParserPosition,
        source: Vec<Token>,
    },

    Comment {
        position: ParserPosition,
        text: String,
    },
    Print {
        position: ParserPosition,
        expr: Box<Token>,
    },

    Sleep {
        position: ParserPosition,
        expr: Box<Token>,
    },

    Constructor {
        position: ParserPosition,
        input: Vec<Token>,
        body: Vec<Token>,
    },

    Function {
        position: ParserPosition,
        function_name: Box<Token>,
        is_static: bool,
        scope: Option<Box<Token>>,
        return_type: Option<Box<Token>>,
        input: Vec<Token>,
        body: Vec<Token>,
    },

    AnonFunction {
        position: ParserPosition,
        input: Vec<Token>,
        body: Vec<Token>,
    },

    Module {
        position: ParserPosition,
        module_name: Box<Token>,
        body: Vec<Token>,
    },

    Class {
        position: ParserPosition,
        class_name: Box<Token>,
        body: Vec<Token>,
    },

    Identifier {
        position: ParserPosition,
        name: String,
    },

    DotChain {
        position: ParserPosition,
        start: Box<Token>,
        chain: Vec<Token>,
    },

    Variable {
        position: ParserPosition,
        name: String,
        as_type: Option<String>,
        value: Option<Box<Token>>,
    },

    Attribute {
        position: ParserPosition,
        name: String,
        as_type: Option<String>,
        value: Option<Box<Token>>,
    },

    Constant {
        position: ParserPosition,
        name: String,
        value: Box<Token>,
    },

    NewObject {
        position: ParserPosition,
        name: Box<Token>,
        input: Vec<Token>,
    },

    Assign {
        position: ParserPosition,
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
        position: ParserPosition,
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
        position: ParserPosition,
        chain: Vec<Token>,
    },
    If {
        position: ParserPosition,
        condition: Box<Token>,
        body: Vec<Token>,
    },
    Else {
        position: ParserPosition,
        body: Vec<Token>,
    },

    Match {
        position: ParserPosition,
        expr: Box<Token>,
        arms: Vec<Token>,
        default: Option<Box<Token>>,
    },

    Case {
        position: ParserPosition,
        condition: Box<Token>,
        body: Vec<Token>,
    },

    DefaultCase {
        position: ParserPosition,
        body: Vec<Token>,
    },

    WhileLoop {
        position: ParserPosition,
        condition: Box<Token>,
        body: Vec<Token>,
    },

    ForEach {
        position: ParserPosition,
        ident: Box<Token>,
        collection: Box<Token>,
        body: Vec<Token>,
    },

    ForI {
        position: ParserPosition,
        ident: Box<Token>,
        start: Box<Token>,
        step: Box<Token>,
        end: Box<Token>,
        body: Vec<Token>,
    },

    Break {
        position: ParserPosition,
    },
    Continue {
        position: ParserPosition,
    },

    Call {
        position: ParserPosition,
        name: Box<Token>,
        input: Vec<Token>,
    },

    Return {
        position: ParserPosition,
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

