use crate::compiler::codegen::syntax::{Syntax, TokenPosition};
use crate::compiler::parser::lexer::lexer::LexerError;
use crate::compiler::parser::lexer::Token;

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
}

#[derive(Debug, PartialEq)]
pub enum IdentifierError {
    InvalidIdentifier(String),
    IdentifierStartsWithNumber(String),
}

#[derive(Debug, PartialEq)]
pub struct CompilerError {
    pub error: CompilerErrorType,
    pub position: TokenPosition,
}

#[derive(Debug, PartialEq)]
pub enum CompilerErrorType {
    ParseError(ParserErrorType),
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