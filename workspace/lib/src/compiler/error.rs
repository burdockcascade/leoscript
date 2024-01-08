use crate::compiler::codegen::syntax::{Syntax, TokenPosition};
use crate::compiler::parser::lexer::lexer::LexerError;
use crate::compiler::parser::lexer::Token;

#[macro_export]
macro_rules! parser_error {
    ($cursor:expr, $error:expr) => {
        Err(ParserError {
            error: $error,
            position: TokenPosition {
                line: $cursor.line,
                column: $cursor.column,
            },
        })
    };
}

#[derive(Debug, PartialEq)]
pub struct ParserError {
    pub error: ParserErrorType,
    pub position: TokenPosition,
}

#[derive(Debug, PartialEq)]
pub enum ParserErrorType {
    InvalidIdentifier(String),
    InvalidParameterName {
        name: String,
        reason: IdentifierError,
    },
    InvalidFunctionName {
        name: String,
        reason: IdentifierError,
    },
    UnrecognizedToken(String),
    InvalidVariableName {
        name: String,
        reason: IdentifierError,
    },
    IdentifierStartsWithNumber(String),
    UnexpectedToken {
        expected: Token,
        found: Token,
    },
    UnwantedToken(Token),
    InvalidArgumentName { name: String, reason: IdentifierError },
    UnexpectedParserError(LexerError),
    NoMatchFound,
    UnexpectedError,
    InvalidLiteral(String),
    InvalidExpressionItem(String),
    InvalidChainedItem(String),
    UnexpectedEnd,
    InvalidArrayAccess,
    InvalidMemberAccess,
    InvalidStaticAccess,
    InvalidNewObject,
    InvalidMapItem(String),
    InvalidImportPath(String),
    InvalidSyntax,
}

#[derive(Debug, PartialEq)]
pub enum IdentifierError {
    InvalidIdentifier(String),
    IdentifierStartsWithNumber(String),
}

#[macro_export]
macro_rules! codegen_error {
    ($cursor:expr, $error:expr) => {
        Err(CodegenError {
            error: $error,
            position: TokenPosition {
                line: $cursor.line,
                column: $cursor.column,
            },
        })
    };
}

#[derive(Debug, PartialEq)]
pub struct CodegenError {
    pub error: CodegenErrorType,
    pub position: TokenPosition,
}

#[derive(Debug, PartialEq)]
pub enum CodegenErrorType {
    NoTokens,

    GlobalNotFound(String),
    VariableNotDeclared(String),
    VariableAlreadyDeclared(String),
    UnableToAssign,
    UnknownParameterToken,

    FeatureNotImplemented,
    UnableToCompile,
    UnableToCompileScript,
    IfStatementInvalid,
    UnrecognizedItem,

    BreakOutsideOfLoop,
    ContinueOutsideOfLoop,

    InvalidChainItem,
    InvalidDefaultCase,
    InvalidMatchArm,

    InvalidImportExpression(String),
    InvalidImportPath(String),
    UnableToImportFile(String),

    UnknownError(String),

    UnableToGetWorkingDirectory,
    UnableToReadFile(String),

    UnableToParseTokens,
    ExpectedBlockEnd,
    NothingToParse,
    InvalidImportReference { position: TokenPosition, name: String },
    InvalidIdentifier { position: TokenPosition, name: String },

    NoInstructionsGenerated,
    NoTokensGenerated,
    UnableToCompileFunction(String),
    UnableToCompileParameterVariable(Syntax),
    UnableToCompileChainItem(Syntax),
    UnableToAssignItem(Box<Syntax>),
    UnableToCreateNewObjectFrom(Box<Syntax>),
    UnableToIterateOver(Box<Syntax>),
    NoIteratorJumpsFound,
    InvalidExpressionItem(Box<Syntax>),
    InvalidIteratorVariable,
    InvalidMatch,
}