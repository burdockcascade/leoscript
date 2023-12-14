use std::collections::HashMap;

use crate::common::instruction::Instruction;
use crate::common::variant::Variant;
use crate::common::warning::ScriptWarning;

#[derive(Debug, PartialEq)]
pub struct Program {
    pub instructions: Vec<Instruction>,
    pub globals: HashMap<String, Variant>,
    pub warnings: Vec<ScriptWarning>
}

impl Default for Program {
    fn default() -> Self {
        Program {
            instructions: Vec::new(),
            globals: HashMap::new(),
            warnings: vec![],
        }
    }
}