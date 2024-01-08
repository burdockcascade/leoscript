use crate::runtime::error::RuntimeError;
use crate::runtime::ir::variant::Variant;

#[derive(Debug, PartialEq)]
pub struct Frame {
    pub return_address: usize,
    pub stack_pointer: usize,
    pub variables: Vec<Variant>,
}

impl Default for Frame {
    fn default() -> Self {
        Frame {
            return_address: 0,
            stack_pointer: 0,
            variables: Vec::with_capacity(16),
        }
    }
}

impl Frame {
    pub fn set_variable(&mut self, index: usize, value: Variant) -> Result<(), RuntimeError> {
        if index >= self.variables.len() {
            self.variables.resize(index + 1, Variant::Null);
        }
        self.variables[index] = value;
        Ok(())
    }

    pub fn get_variable(&self, index: usize) -> Result<&Variant, RuntimeError> {
        if index >= self.variables.len() {
            return Err(RuntimeError::InvalidVariableIndex(index));
        }
        Ok(&self.variables[index])
    }
}