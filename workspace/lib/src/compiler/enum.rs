use std::collections::HashMap;

use log::trace;

use crate::common::error::ScriptError;
use crate::common::variant::Variant;
use crate::compiler::token::{Token, TokenPosition};

pub fn compile_enum(_position: TokenPosition, name: String, items: Vec<Token>) -> Result<Variant, ScriptError> {
    trace!("Compiling enum: {}", name);

    let mut enum_def = HashMap::default();

    let mut index = 0;

    for item in items {
        enum_def.insert(item.to_string(), index);
        index += 1;
    }

    Ok(Variant::Enum(enum_def))
}