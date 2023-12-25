use std::time::Duration;
use leoscript_runtime::ir::program::Program;
use crate::compiler::script::compile_script;
use crate::parser::token::TokenPosition;

mod class;
mod r#enum;
mod function;
mod module;
mod script;
mod variable;

#[derive(Debug, PartialEq)]
struct CompilerError {
    pub error: CompilerErrorType,
    pub position: TokenPosition,
}

#[derive(Debug, PartialEq)]
struct CompilerWarning {
    pub warning: CompilerWarningType,
    pub position: TokenPosition,
}

#[derive(Debug, PartialEq)]
pub enum CompilerWarningType {
    ImportFileEmpty(String),
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

    InvalidIdentifier,
    UnableToGetWorkingDirectory,
    UnableToReadFile(String),
}

#[derive(Debug, PartialEq)]
pub struct CompilerResult {
    pub program: Program,
    pub compile_time: Duration,
    pub parser_time: Duration,
    pub source_files: Vec<String>,
}

pub fn compile_program(source: &str) -> Result<CompilerResult, CompilerError> {

    // compile master script
    let script = compile_script(source, 0)?;

    // return script result
    Ok(CompilerResult {
        program: Program {
            instructions: script.instructions,
            globals: script.globals,
        },
        compile_time: script.compiler_time,
        parser_time: script.parser_time,
        source_files: script.imports
    })
}