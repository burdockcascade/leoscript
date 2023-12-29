use crate::runtime::error::RuntimeError;
use crate::runtime::ir::program::Program;
use crate::runtime::ir::variant::Variant;
use crate::runtime::vm::thread::{ExecutionResult, Thread};

mod stdlib;
mod vm;
pub mod ir;
pub mod error;

pub fn run_program(program: Program, entrypoint: &str, parameters: Option<Vec<Variant>>) -> Result<ExecutionResult, RuntimeError> {
    let mut thread = Thread::load_program(program)?;

    stdlib::add_standard_library(&mut thread)?;

    thread.run(entrypoint, parameters)
}
