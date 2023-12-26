use crate::runtime::ir::stacktrace::StackTrace;

#[derive(Clone, Debug, PartialEq)]
pub enum Instruction {
    NoOperation,

    SetVariableBuffer(usize),

    // Stack
    PushNull,
    PushInteger(i64),
    PushFloat(f64),
    PushBool(bool),
    PushString(String),
    PushFunctionRef(String),
    PushFunctionPointer(usize),

    // Stack Trace
    PushStackTrace(StackTrace),
    PopStackTrace,

    // Variables
    MoveToLocalVariable(usize),
    LoadLocalVariable(usize),

    // Global
    LoadGlobal(String),
    LoadClass(String),
    LoadMember(String),

    // Objects
    CreateObject,

    // Collections
    GetCollectionItem,
    SetCollectionItem,
    CreateCollectionAsDictionary(usize),
    CreateCollectionAsArray(usize),

    // Iteration
    IteratorInit,
    IteratorNext(usize),

    // Instructions
    Call(usize),
    JumpForward(usize),
    JumpBackward(usize),
    JumpForwardIfFalse(usize),

    // Return
    Return,
    ReturnWithValue,

    // Operators
    Equal,
    NotEqual,
    Add,
    Sub,
    Multiply,
    Divide,
    Pow,

    Not,
    And,
    Or,

    // Comparison
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,

    Print,
    Sleep,

    // Halt Program
    Halt(String),
}