use std::time::Duration;

use crate::compiler::codegen::generate_program;
use crate::compiler::error::{CompilerError, CompilerErrorType};
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

    let Ok(parse_result) = Parser::parse(source) else {
        panic!("Failed to parse source")
    };

    let Ok(compiler_result) = generate_program(parse_result.syntax_tree) else {
        panic!("Failed to generate program")
    };

    Ok(CompilerResult {
        program: compiler_result.program,
        compile_time: compiler_result.codegen_time,
        parser_time: parse_result.parser_time,
        source_files: vec![],
    })
}