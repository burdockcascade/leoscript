use crate::common::variant::Variant;

pub mod error;
pub mod variant;
pub mod instruction;
pub mod program;
pub mod counter;
pub mod stacktrace;

pub type NativeFunctionType = fn(Vec<Variant>) -> Option<Variant>;
