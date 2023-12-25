use std::time::Duration;
use leoscript_runtime::ir::program::Program;
use crate::codegen::generate_program;
use crate::error::CompilerError;
use crate::parser::token::TokenPosition;

mod parser;
pub mod codegen;
pub mod error;
pub mod warning;

#[derive(Debug, PartialEq)]
pub struct CompilerResult {
    pub program: Program,
    pub compile_time: Duration,
    pub parser_time: Duration,
    pub source_files: Vec<String>,
}

pub fn compile(source: &str) -> Result<CompilerResult, CompilerError> {
    generate_program(source)
}