use std::collections::HashMap;

use crate::common::error::ScriptError;
use crate::common::variant::Variant;
use crate::parser::token::{Token, TokenPosition};

pub fn compile_enum(_position: TokenPosition, _name: String, items: Vec<Token>) -> Result<Variant, ScriptError> {
    let mut enum_def = HashMap::default();

    let mut index = 0;

    for item in items {
        enum_def.insert(item.to_string(), index);
        index += 1;
    }

    Ok(Variant::Enum(enum_def))
}