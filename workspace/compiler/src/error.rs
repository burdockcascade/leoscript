use crate::parser::token::TokenPosition;

#[derive(Debug, PartialEq)]
pub struct CompilerError {
    pub error: CompilerErrorType,
    pub position: TokenPosition,
}

#[derive(Debug, PartialEq)]
pub enum CompilerErrorType {
    ParseError,
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

}