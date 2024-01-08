use crate::compiler::error::{CodegenError, ParserError};
use crate::runtime::error::RuntimeError;

#[derive(Debug, PartialEq)]
pub enum ScriptError {

    #[cfg(feature = "compiler")]
    ParserError(ParserError),

    #[cfg(feature = "compiler")]
    CodegenError(CodegenError),

    #[cfg(feature = "runtime")]
    RuntimeError(RuntimeError)
}





