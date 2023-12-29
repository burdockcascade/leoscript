use std::time::Duration;

use crate::compiler::codegen::generate_program;
use crate::compiler::error::CompilerError;
use crate::runtime::ir::program::Program;

pub mod codegen;
pub mod warning;
pub mod error;
mod parser;
mod tokenizer;

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