use crate::compiler::codegen::script::generate_script;
use crate::compiler::CompilerResult;
use crate::compiler::codegen::syntax::TokenPosition;
use crate::compiler::error::{CompilerError, CompilerErrorType};
use crate::runtime::ir::program::Program;

mod class;
mod r#enum;
mod function;
mod module;
pub mod script;
mod variable;
pub mod syntax;

pub fn generate_program(source: &str) -> Result<CompilerResult, CompilerError> {
    let script = generate_script(source, 0)?;

    if script.instructions.len() == 0 {
        return Err(CompilerError {
            error: CompilerErrorType::NoInstructionsGenerated,
            position: TokenPosition::default(),
        });
    }

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
