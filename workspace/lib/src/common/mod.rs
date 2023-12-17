use crate::common::error::ScriptError;
use crate::common::variant::Variant;

pub mod error;
pub mod variant;
pub mod instruction;
pub mod program;
pub mod counter;
pub mod stacktrace;
pub mod warning;

pub type NativeFunctionType = fn(Vec<Variant>) -> Result<Option<Variant>, ScriptError>;
