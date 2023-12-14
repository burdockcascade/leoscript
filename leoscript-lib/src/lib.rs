use crate::common::error::ScriptError;
use crate::common::variant::Variant;
use crate::compiler::script::compile_program;
use crate::stdlib::add_standard_library;
use crate::vm::thread::Thread;

mod compiler;
mod vm;
pub mod common;
mod stdlib;

#[derive(Debug)]
pub struct ScriptResult {
    pub result: Option<Variant>,
}

pub fn run_script(source: &str, entrypoint: &str, parameters: Option<Vec<Variant>>) -> Result<ScriptResult, ScriptError> {
    let program = compile_program(source)?;

    let mut t = Thread::load_program(program)?;

    add_standard_library(&mut t)?;

    let output = t.run(entrypoint, parameters);

    match output {
        Ok(v) => Ok(ScriptResult {
            result: v,
        }),
        Err(e) => Err(e),
    }
}