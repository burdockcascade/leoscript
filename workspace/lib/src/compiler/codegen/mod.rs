use std::collections::HashMap;
use std::time::Duration;

use crate::compiler::codegen::script::generate_script;
use crate::compiler::codegen::syntax::TokenPosition;
use crate::compiler::CompilerResult;
use crate::compiler::error::{CompilerError, CompilerErrorType};
use crate::compiler::warning::CompilerWarning;
use crate::runtime::ir::instruction::Instruction;
use crate::runtime::ir::program::Program;
use crate::runtime::ir::variant::Variant;

pub mod script;
pub mod syntax;
mod function;

#[macro_export]
macro_rules! compiler_error {
    ($cursor:expr, $error:expr) => {
        Err(CompilerError {
            error: $error,
            position: TokenPosition {
                line: $cursor.line,
                column: $cursor.column,
            },
        })
    };
}

pub struct CodeGenerationResult {
    pub instructions: Vec<Instruction>,
    pub globals: HashMap<String, Variant>,
    pub compiler_time: Duration,
    pub parser_time: Duration,
    pub imports: Vec<String>,
    pub warnings: Vec<CompilerWarning>,
}

impl Default for CodeGenerationResult {
    fn default() -> Self {
        CodeGenerationResult {
            instructions: Default::default(),
            globals: Default::default(),
            compiler_time: Default::default(),
            parser_time: Default::default(),
            imports: Default::default(),
            warnings: Default::default(),
        }
    }
}

pub fn generate_program(source: &str) -> Result<CompilerResult, CompilerError> {
    let cgr = generate_script(source, 0)?;

    if cgr.instructions.len() == 0 {
        return Err(CompilerError {
            error: CompilerErrorType::NoInstructionsGenerated,
            position: TokenPosition::default(),
        });
    }

    Ok(CompilerResult {
        program: Program {
            instructions: cgr.instructions,
            globals: cgr.globals,
        },
        compile_time: cgr.compiler_time,
        parser_time: cgr.parser_time,
        source_files: cgr.imports,
    })
}
