use std::time::Duration;

use crate::common::error::ScriptError;
use crate::common::program::Program;
use crate::common::variant::Variant;
use crate::common::warning::ScriptWarning;
use crate::compiler::{compile_program, CompilerResult};
use crate::stdlib::add_standard_library;
use crate::vm::thread::Thread;

mod compiler;
mod vm;
pub mod common;
mod stdlib;

#[derive(Debug)]
pub struct ScriptResult {
    pub result: Option<Variant>,
    pub warnings: Vec<ScriptWarning>,
    pub parser_time: Duration,
    pub compile_time: Duration,
    pub execution_time: Duration,
    pub total_time: Duration,
}

pub fn compile_script(source: &str) -> Result<CompilerResult, ScriptError> {
    compile_program(source)
}

pub fn execute_program(program: Program, entrypoint: &str, parameters: Option<Vec<Variant>>) -> Result<ScriptResult, ScriptError> {
    let mut t = Thread::load_program(program)?;
    add_standard_library(&mut t)?;
    let execution_result = t.run(entrypoint, parameters)?;
    Ok(ScriptResult {
        result: execution_result.output,
        warnings: Vec::new(),
        parser_time: Duration::default(),
        compile_time: Duration::default(),
        execution_time: execution_result.execution_time,
        total_time: execution_result.execution_time,
    })
}

pub fn run_script(source: &str, entrypoint: &str, parameters: Option<Vec<Variant>>) -> Result<ScriptResult, ScriptError> {
    let compiler_result = compile_program(source)?;

    let mut t = Thread::load_program(compiler_result.program)?;

    add_standard_library(&mut t)?;

    let execution_result = t.run(entrypoint, parameters)?;

    Ok(ScriptResult {
        result: execution_result.output,
        warnings: compiler_result.warnings,
        parser_time: compiler_result.parser_time,
        compile_time: compiler_result.compile_time,
        execution_time: execution_result.execution_time,
        total_time: compiler_result.parser_time + compiler_result.compile_time + execution_result.execution_time,
    })
}