use leoscript_runtime::ir::program::Program;
use crate::codegen::script::generate_script;
use crate::{CompilerResult};
use crate::error::CompilerError;
use crate::parser::token::TokenPosition;

mod class;
mod r#enum;
mod function;
mod module;
pub mod script;
mod variable;

pub fn generate_program(source: &str) -> Result<CompilerResult, CompilerError> {
    let script = generate_script(source, 0)?;

    Ok(CompilerResult {
        program: Program {
            instructions: script.instructions,
            globals: script.globals,
        },
        compile_time: script.compiler_time,
        parser_time: script.parser_time,
        source_files: script.imports,
    })
}