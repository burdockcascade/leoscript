use std::time::Duration;

use crate::compiler::codegen::generate_program;
use crate::compiler::error::{CodegenError, CodegenErrorType, ParserError};
use crate::compiler::parser::Parser;
use crate::error::ScriptError;
use crate::runtime::ir::program::Program;

pub mod codegen;
pub mod warning;
pub mod error;
pub mod parser;

#[derive(Debug, PartialEq)]
pub struct CompilerResult {
    pub program: Program,
    pub compile_time: Duration,
    pub parser_time: Duration,
    pub source_files: Vec<String>,
}

pub fn compile(source: &str) -> Result<CompilerResult, ScriptError> {

    let parse_result = match Parser::parse(source) {
        Ok(pr) => pr,
        Err(e) => return Err(ScriptError::ParserError(e)),
    };

    match generate_program(parse_result.syntax_tree) {
        Ok(cr) => Ok(CompilerResult {
            program: cr.program,
            compile_time: cr.codegen_time,
            parser_time: parse_result.parser_time,
            source_files: vec![],
        }),
        Err(e) => Err(ScriptError::CodegenError(e)),
    }
}