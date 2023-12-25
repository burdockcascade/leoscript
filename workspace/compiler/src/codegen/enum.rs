use std::collections::HashMap;
use leoscript_runtime::ir::variant::Variant;
use crate::error::CompilerError;

use crate::parser::token::{Token, TokenPosition};

pub fn generate_enum(_position: TokenPosition, _name: String, items: Vec<Token>) -> Result<Variant, CompilerError> {
    let mut enum_def = HashMap::default();

    let mut index = 0;

    for item in items {
        enum_def.insert(item.to_string(), index);
        index += 1;
    }

    Ok(Variant::Enum(enum_def))
}