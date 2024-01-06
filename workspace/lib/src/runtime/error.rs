use crate::runtime::ir::instruction::Instruction;
use crate::runtime::ir::variant::Variant;

#[derive(Debug, PartialEq)]
pub enum RuntimeError {
    NoInstructions,

    ExpectedValueOnStack,
    ExpectedClassOnStack,
    ExpectedIntegerOnStack,
    ExpectedIteratorOnStack,
    ExpectedObjectOnStack,
    InvalidFunctionOnStack(Variant),

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

    UnknownNativeParameterToken,
    InvalidNativeFunction(String),
    ExpectedSelfForNativeFunction,
    ObjectIndexNotFound(String),
    ModuleIndexNotFound(String),
    EnumIndexNotFound(String),
    ExpectedMemberNameOnStack,
    InvalidCollectionKey(Variant),
    InvalidVariableIndex(usize),
    InfiniteLoop,
}