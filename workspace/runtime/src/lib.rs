use crate::error::RuntimeError;
use crate::ir::program::Program;
use crate::ir::variant::Variant;
use crate::vm::thread::{ExecutionResult, Thread};

mod error;
mod stdlib;
mod ir;
mod vm;

fn run_program(program: Program, entrypoint: &str, parameters: Option<Vec<Variant>>) -> Result<ExecutionResult, RuntimeError> {
    let mut thread = Thread::load_program(program)?;
    thread.run(entrypoint, parameters)
}
