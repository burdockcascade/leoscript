use std::time::Duration;

use crate::compiler::codegen::script::Script;
use crate::compiler::codegen::syntax::Syntax;
use crate::compiler::error::CodegenError;
use crate::compiler::warning::CompilerWarning;
use crate::runtime::ir::program::Program;

pub mod script;
pub mod syntax;
mod function;

#[derive(Debug)]
pub struct CodeGenerationResult {
    pub program: Program,
    pub codegen_time: Duration,
    pub warnings: Vec<CompilerWarning>,
}

pub fn generate_program(source: Vec<Syntax>) -> Result<CodeGenerationResult, CodegenError> {

    let compiler_timer = std::time::Instant::now();

    let mut s = Script::new();
    s.generate_script(source)?;

    Ok(CodeGenerationResult {
        program: Program {
            instructions: s.instructions,
            globals: s.structure,
        },
        codegen_time: compiler_timer.elapsed(),
        warnings: vec![],
    })
}
