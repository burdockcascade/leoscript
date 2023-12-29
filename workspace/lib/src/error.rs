use crate::compiler::error::{CompilerError, ParserError};
use crate::runtime::error::RuntimeError;

#[derive(Debug, PartialEq)]
pub enum ScriptError {

    #[cfg(feature = "compiler")]
    ParserError(ParserError),

    #[cfg(feature = "compiler")]
    CompilerError(CompilerError),

    #[cfg(feature = "runtime")]
    RuntimeError(RuntimeError)
}





