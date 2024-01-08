use std::time::Duration;

use crate::compiler::compile;
use crate::error::ScriptError;
use crate::runtime::ir::variant::Variant;
use crate::runtime::run_program;

pub mod compiler;
pub mod runtime;
pub mod error;

#[derive(Debug, PartialEq)]
pub struct ScriptResult {
    pub result: Option<Variant>,
    pub parser_time: Duration,
    pub compile_time: Duration,
    pub execution_time: Duration,
    pub imports: Vec<String>,
    pub total_time: Duration,
}

pub fn run_script(source: &str, entrypoint: &str, args: Option<Vec<Variant>>) -> Result<ScriptResult, ScriptError> {
    let compiler_result = match compile(source) {
        Ok(result) => result,
        Err(e) => return Err(e),
    };

    match run_program(compiler_result.program, entrypoint, args) {
        Ok(execution_result) => Ok(ScriptResult {
            result: execution_result.output,
            parser_time: compiler_result.parser_time,
            compile_time: compiler_result.compile_time,
            execution_time: execution_result.execution_time,
            imports: compiler_result.source_files,
            total_time: compiler_result.parser_time + compiler_result.compile_time + execution_result.execution_time,
        }),
        Err(e) => Err(ScriptError::RuntimeError(e))
    }
}
