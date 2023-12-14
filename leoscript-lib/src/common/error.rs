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
    ($error:expr) => {
        Err(ScriptError::CompilerError {
            error: $error,
            line: 0,
            column: 0,
        })
    };
}

#[macro_export]
macro_rules! script_parse_error {
    ($error:expr, $position:expr) => {
        Err(ScriptError::ParserError {
            error: $error,
            line: $position.line,
            column: $position.column,
        })
    };
    ($error:expr) => {
        Err(ScriptError::ParserError {
            error: $error,
            line: 0,
            column: 0,
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

#[macro_export]
macro_rules! script_system_error {
    ($error:expr) => {
        Err(ScriptError::SystemError {
            error: $error,
        })
    };
}

#[macro_export]
macro_rules! script_native_function_error {
    ($error:expr) => {
        Err(ScriptError::NativeFunctionError {
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
        error: ParseError,
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
    NativeFunctionError {
        error: NativeFunctionError,
    },
}

#[derive(Debug, PartialEq)]
pub enum NativeFunctionError {
    InvalidSelf,
    UnknownParameterToken,
    InvalidInternalValue,
    InvalidNativeFunction(String),
}

#[derive(Debug, PartialEq)]
pub enum SystemError {
    ExpectedNativeFunction,
    UnableToGetLocalDirectory
}

#[derive(Debug, PartialEq)]
pub enum ParseError {
    UnableToParseTokens,
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
    UnableToCompileScript,
    IfStatementInvalid,
    UnrecognizedItem,

    BreakOutsideOfLoop,
    ContinueOutsideOfLoop,

    InvalidChainItem,
    InvalidDefaultCase,
    InvalidMatchArm,

    InvalidImportExpression(String),
    InvalidImportPath(String),
    UnableToImportFile(String),

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