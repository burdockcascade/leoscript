use std::collections::HashMap;

use crate::common::instruction::Instruction;
use crate::common::variant::Variant;

#[derive(Debug, PartialEq)]
pub struct Program {
    pub instructions: Vec<Instruction>,
    pub globals: HashMap<String, Variant>,
}

impl Default for Program {
    fn default() -> Self {
        Program {
            instructions: Vec::new(),
            globals: HashMap::new(),
        }
    }
}