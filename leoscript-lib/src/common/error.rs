use crate::common::instruction::Instruction;
use crate::common::stacktrace::StackTrace;
use crate::common::variant::Variant;

#[macro_export]
macro_rules! script_compile_error {
    ($error:expr, $position:expr) => {
        Err(ScriptError::CompilerError {
            error: $error,
            line: $position.line,
            column: $position.column,
        })
    };
}

#[macro_export]
macro_rules! script_runtime_error {
    ($trace:ident, $error:expr) => {
        Err(ScriptError::RuntimeError {
            trace: Some($trace),
            error: $error,
        })
    };
}

#[derive(Debug, PartialEq)]
pub enum ScriptError {
    SystemError {
        error: SystemError,
    },
    ParserError {
        line: usize,
        column: usize,
    },
    CompilerError {
        error: CompilerError,
        line: usize,
        column: usize,
    },
    RuntimeError {
        trace: Option<Vec<StackTrace>>,
        error: RuntimeError,
    },
}

#[derive(Debug, PartialEq)]
pub enum SystemError {
    ExpectedNativeFunction,
}

#[derive(Debug, PartialEq)]
pub enum ParseError {
    ExpectedBlockEnd,
}

#[derive(Debug, PartialEq)]
pub enum CompilerError {
    ParseError,
    NoTokens,

    GlobalNotFound(String),
    VariableNotDeclared(String),
    VariableAlreadyDeclared(String),
    UnableToAssign,
    UnknownParameterToken,

    FeatureNotImplemented,
    UnableToCompile,
    IfStatementInvalid,
    UnrecognizedItem,

    BreakOutsideOfLoop,
    ContinueOutsideOfLoop,

    InvalidChainItem

}

#[derive(Debug, PartialEq)]
pub enum RuntimeError {

    NoInstructions,

    ExpectedValueOnStack,
    ExpectedClassOnStack,
    ExpectedIntegerOnStack,
    ExpectedIteratorOnStack,
    ExpectedObjectOnStack,

    InstructionPointerOutOfBounds(usize),
    EntryPointNotFound(String),
    GlobalNotFound(String),
    InstructionNotImplemented(Instruction),
    FunctionNotFound(String),
    MethodNotFound(String),
    NativeFunctionNotFound(String),

    InvalidFrame,
    InvalidStackIndex { index: usize, size: usize },
    InvalidFunctionRef(String),
    InvalidCallDestination(Variant),
    InvalidFunctionPointer,
    InvalidClassTemplate,
    InvalidFramePointer,
    InvalidReturnPointer,
    InvalidIteratorStep,
    InvalidIteratorStart,
    InvalidIteratorNext,
    InvalidObjectOnStack,
    InvalidDictionaryKey,
    InvalidArrayIndex,
    InvalidObjectMember,
    InvalidModuleMember,
    InvalidEnumItem,
    InvalidDictionaryItems,
    InvalidCollection,
}