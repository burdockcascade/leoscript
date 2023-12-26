use std::collections::HashMap;
use crate::runtime::ir::instruction::Instruction;
use crate::runtime::ir::variant::Variant;


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