use crate::compiler::codegen::function::Function;
use crate::compiler::error::CompilerError;
use crate::runtime::ir::instruction::Instruction;
use crate::runtime::ir::stacktrace::StackTrace;

impl Function {
    pub(crate) fn generate_stack_trace_push(&mut self, line: usize) -> Result<(), CompilerError> {
        self.instructions.push(Instruction::PushStackTrace(StackTrace {
            line,
            file: String::from("unknown"),
            function: self.name.clone(),
        }));

        Ok(())
    }

    pub(crate) fn generate_stack_trace_pop(&mut self) -> Result<(), CompilerError> {
        self.instructions.push(Instruction::PopStackTrace);
        Ok(())
    }
}