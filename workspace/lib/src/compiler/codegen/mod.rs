use std::collections::HashMap;
use std::time::Duration;
use crate::compiler::codegen::script::Script;
use crate::compiler::codegen::syntax::Syntax;

use crate::compiler::CompilerResult;
use crate::compiler::error::{CompilerError};
use crate::compiler::warning::CompilerWarning;
use crate::runtime::ir::instruction::Instruction;
use crate::runtime::ir::program::Program;
use crate::runtime::ir::variant::Variant;

pub mod script;
pub mod syntax;
mod function;

#[derive(Debug)]
pub struct CodeGenerationResult {
    pub program: Program,
    pub codegen_time: Duration,
    pub warnings: Vec<CompilerWarning>,
}

pub fn generate_program(source: Vec<Syntax>) -> Result<CodeGenerationResult, CompilerError> {

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
