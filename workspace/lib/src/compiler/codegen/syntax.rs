use std::collections::HashMap;
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq)]
pub enum Syntax {

    Module {
        position: TokenPosition,
        module_name: Box<Syntax>,
        constants: Vec<Syntax>,
        functions: Vec<Syntax>,
        classes: Vec<Syntax>,
        enums: Vec<Syntax>,
        modules: Vec<Syntax>,
        imports: Vec<Syntax>,
    },

    Import {
        position: TokenPosition,
        source: Vec<Syntax>,
    },

    Comment {
        position: TokenPosition,
        text: String,
    },
    Print {
        position: TokenPosition,
        expr: Box<Syntax>,
    },

    Sleep {
        position: TokenPosition,
        expr: Box<Syntax>,
    },

    Constructor {
        position: TokenPosition,
        input: Vec<Syntax>,
        body: Vec<Syntax>,
    },

    Function {
        position: TokenPosition,
        function_name: Box<Syntax>,
        is_static: bool,
        scope: Option<Box<Syntax>>,
        return_type: Option<Box<Syntax>>,
        parameters: Vec<Syntax>,
        body: Vec<Syntax>,
    },

    AnonFunction {
        position: TokenPosition,
        input: Vec<Syntax>,
        body: Vec<Syntax>,
    },

    Class {
        position: TokenPosition,
        class_name: Box<Syntax>,
        attributes: Vec<Syntax>,
        constructor: Option<Box<Syntax>>,
        methods: Vec<Syntax>
    },

    Identifier {
        position: TokenPosition,
        name: String,
    },

    Variable {
        position: TokenPosition,
        name: Box<Syntax>,
        as_type: Option<String>,
        value: Option<Box<Syntax>>,
    },

    Attribute {
        position: TokenPosition,
        name: Box<Syntax>,
        as_type: Option<Box<Syntax>>,
        value: Option<Box<Syntax>>,
    },

    Constant {
        position: TokenPosition,
        name: String,
        value: Box<Syntax>,
    },

    NewObject {
        position: TokenPosition,
        name: Box<Syntax>,
        input: Vec<Syntax>,
    },

    Assign {
        position: TokenPosition,
        target: Box<Syntax>,
        value: Box<Syntax>,
    },

    Null,
    Integer(i64),
    Float(f64),
    Bool(bool),
    String(String),

    Array(Vec<Syntax>),
    Dictionary(HashMap<String, Syntax>),

    CollectionIndex(Box<Syntax>),

    Enum {
        position: TokenPosition,
        name: Box<Syntax>,
        items: Vec<Syntax>,
    },

    Not { expr: Box<Syntax> },
    And { expr1: Box<Syntax>, expr2: Box<Syntax> },
    Or { expr1: Box<Syntax>, expr2: Box<Syntax> },
    Eq { expr1: Box<Syntax>, expr2: Box<Syntax> },
    Ne { expr1: Box<Syntax>, expr2: Box<Syntax> },
    Lt { expr1: Box<Syntax>, expr2: Box<Syntax> },
    Le { expr1: Box<Syntax>, expr2: Box<Syntax> },
    Gt { expr1: Box<Syntax>, expr2: Box<Syntax> },
    Ge { expr1: Box<Syntax>, expr2: Box<Syntax> },
    Add { expr1: Box<Syntax>, expr2: Box<Syntax> },
    Sub { expr1: Box<Syntax>, expr2: Box<Syntax> },
    Mul { expr1: Box<Syntax>, expr2: Box<Syntax> },
    Div { expr1: Box<Syntax>, expr2: Box<Syntax> },
    Pow { expr1: Box<Syntax>, expr2: Box<Syntax> },

    IfChain {
        position: TokenPosition,
        arms: Vec<Syntax>,
    },
    If {
        position: TokenPosition,
        condition: Box<Syntax>,
        body: Vec<Syntax>,
    },
    Else {
        position: TokenPosition,
        body: Vec<Syntax>,
    },

    Match {
        position: TokenPosition,
        expr: Box<Syntax>,
        arms: Vec<Syntax>,
        default: Option<Box<Syntax>>,
    },
    Case {
        position: TokenPosition,
        condition: Box<Syntax>,
        body: Vec<Syntax>,
    },
    DefaultCase {
        position: TokenPosition,
        body: Vec<Syntax>,
    },

    WhileLoop {
        position: TokenPosition,
        condition: Box<Syntax>,
        body: Vec<Syntax>,
    },

    ForEach {
        position: TokenPosition,
        ident: Box<Syntax>,
        collection: Box<Syntax>,
        body: Vec<Syntax>,
    },

    Range {
        start: Box<Syntax>,
        end: Box<Syntax>,
        step: Option<Box<Syntax>>
    },

    Break {
        position: TokenPosition,
    },
    Continue {
        position: TokenPosition,
    },

    Call {
        position: TokenPosition,
        target: Box<Syntax>,
        args: Vec<Syntax>,
    },

    Return {
        position: TokenPosition,
        expr: Option<Box<Syntax>>,
    },

    Property {
        position: TokenPosition,
        name: Box<Syntax>
    },
    ArrayAccess {
        position: TokenPosition,
        target: Box<Syntax>,
        index: Box<Syntax>
    },
    MemberAccess {
        position: TokenPosition,
        target: Box<Syntax>,
        index: Box<Syntax>
    },
    StaticAccess { position: TokenPosition, target: Box<Syntax>, index: Box<Syntax> },
}

pub enum Visibility {
    Public,
    Private,
}

impl Display for Syntax {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Syntax::Identifier { name, .. } => name.to_string(),
            Syntax::Variable { name, .. } => name.to_string(),
            Syntax::String(s) => s.to_string(),
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

