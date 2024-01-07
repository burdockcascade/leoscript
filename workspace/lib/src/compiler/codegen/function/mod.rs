use std::collections::HashMap;

use crate::compiler::codegen::syntax::{Syntax, TokenPosition};
use crate::compiler::error::CompilerError;
use crate::runtime::ir::instruction::Instruction;

mod expression;
mod stacktrace;
mod branching;
mod looping;
mod statements;
mod variables;

#[derive(Clone)]
pub struct Function {
    name: String,
    pub instructions: Vec<Instruction>,
    pub variables: HashMap<String, Variable>,
    pub globals: HashMap<String, Variable>,
    pub anon_functions: HashMap<String, Vec<Instruction>>,
    iterators: Vec<IteratorTracker>,
}

impl Default for Function {
    fn default() -> Self {
        Function {
            name: Default::default(),
            instructions: Default::default(),
            variables: Default::default(),
            globals: Default::default(),
            anon_functions: Default::default(),
            iterators: Default::default(),
        }
    }
}

#[derive(Clone)]
pub struct Variable {
    pub slot_index: usize,
    pub name: String,
}

#[derive(Clone)]
struct IteratorTracker {
    breaks: Vec<usize>,
    continues: Vec<usize>,
}

impl Function {
    pub fn new(position: TokenPosition, name: String, parameters: Vec<Syntax>, body: Vec<Syntax>) -> Result<Self, CompilerError> {
        let mut f = Function {
            name,
            instructions: vec![],
            variables: Default::default(),
            globals: Default::default(),
            anon_functions: Default::default(),
            iterators: vec![],
        };

        f.generate_stack_trace_push(position.line)?;

        // store the parameters as variables
        f.generate_parameters(parameters.clone())?;

        // compile the statements
        f.generate_statements(body)?;

        // if tha last instruction is not a return then add one
        match f.instructions.last() {
            Some(Instruction::Return { .. }) => {}
            _ => f.instructions.push(Instruction::Return { with_value: false })
        }

        f.generate_stack_trace_pop()?;

        Ok(f)
    }
}

