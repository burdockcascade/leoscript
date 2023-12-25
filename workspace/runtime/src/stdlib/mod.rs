use crate::error::RuntimeError;
use crate::ir::variant::Variant;


pub type NativeFunctionType = fn(Vec<Variant>) -> Result<Option<Variant>, RuntimeError>;