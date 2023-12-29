use std::collections::HashMap;

use crate::compiler::codegen::syntax::{Syntax, TokenPosition};
use crate::compiler::error::CompilerError;
use crate::runtime::ir::variant::Variant;

pub fn generate_enum(_position: TokenPosition, _name: String, items: Vec<Syntax>) -> Result<Variant, CompilerError> {
    let mut enum_def = HashMap::default();

    let mut index = 0;

    for item in items {
        enum_def.insert(item.to_string(), index);
        index += 1;
    }

    Ok(Variant::Enum(enum_def))
}