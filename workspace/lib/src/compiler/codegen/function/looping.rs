use log::trace;

use crate::compiler::codegen::function::{Function, IteratorTracker};
use crate::compiler::codegen::syntax::{Syntax, TokenPosition};
use crate::compiler::error::{CodegenError, CodegenErrorType};
use crate::runtime::ir::instruction::Instruction;

impl Function {
    pub(crate) fn generate_iterator(&mut self, position: TokenPosition, var: Box<Syntax>, target: Box<Syntax>, block: Vec<Syntax>) -> Result<(), CodegenError> {
        self.iterators.push(IteratorTracker {
            breaks: vec![],
            continues: vec![],
        });

        match *var {
            Syntax::Identifier { .. } => {
                self.generate_variable_with_value(position, var.clone(), None)?;
            }
            _ => return Err(CodegenError {
                error: CodegenErrorType::InvalidIteratorVariable,
                position,
            })
        }

        // get variable slot
        let var_slot = self.variables.get(var.to_string().as_str()).unwrap().slot_index;

        match *target {
            Syntax::Range { start, end, step } => {

                // compile target
                self.generate_expression(position, end)?;

                // compile counter step
                match step {
                    Some(step) => self.generate_expression(position, step)?,
                    None => self.instructions.push(Instruction::PushInteger(1))
                }

                // compile counter start
                self.generate_expression(position, start)?;
            }
            _ => {

                // compile target
                self.generate_expression(position, target)?;

                // compile counter step
                self.instructions.push(Instruction::PushInteger(1));

                // compile counter start
                self.instructions.push(Instruction::PushInteger(0));
            }
        }

        // Create Iterator
        self.instructions.push(Instruction::IteratorInit);

        // temp jump to end
        let start_ins_ptr = self.instructions.len();
        self.instructions.push(Instruction::IteratorNext(var_slot));
        self.instructions.push(Instruction::JumpForwardIfFalse(0));

        // compile statements inside loop block
        self.generate_statements(block)?;

        // jump back to start
        self.instructions.push(Instruction::JumpBackward(self.instructions.len() - start_ins_ptr));

        self.update_iterator_jumps(position, start_ins_ptr)?;

        self.instructions[start_ins_ptr + 1] = Instruction::JumpForwardIfFalse(self.instructions.len() - start_ins_ptr - 1);

        Ok(())
    }

    // compile while loop
    pub(crate) fn generate_while_loop(&mut self, position: TokenPosition, expr: Box<Syntax>, block: Vec<Syntax>) -> Result<(), CodegenError> {
        self.iterators.push(IteratorTracker {
            breaks: vec![],
            continues: vec![],
        });

        // Mark instruction pointer
        let start_ins_ptr = self.instructions.len();

        // Compile expression
        self.generate_expression(position, expr)?;

        // Jump to end if expression is false
        let jump_not_true = self.instructions.len();
        self.instructions.push(Instruction::Halt(String::from("no jump-not-true provided")));

        // Compile statements inside loop block
        self.generate_statements(block)?;

        // Goto loop start
        self.instructions.push(Instruction::JumpBackward(self.instructions.len() - start_ins_ptr));

        // Update jump not true value
        let jump_to_pos = self.instructions.len() - jump_not_true;
        self.instructions[jump_not_true] = Instruction::JumpForwardIfFalse(jump_to_pos);

        self.update_iterator_jumps(position, start_ins_ptr)?;

        Ok(())
    }

    fn update_iterator_jumps(&mut self, position: TokenPosition, start_ins_ptr: usize) -> Result<(), CodegenError> {
        // get iterator
        let Some(it) = self.iterators.pop() else {
            return Err(CodegenError {
                error: CodegenErrorType::NoIteratorJumpsFound,
                position,
            });
        };

        // update jump to end
        it.breaks.iter().for_each(|ip| {
            trace!("updating jump to end: {}", ip);
            self.instructions[*ip] = Instruction::JumpForward(self.instructions.len() - ip);
        });

        // update jump to start
        it.continues.iter().for_each(|ip| {
            trace!("updating jump to start: {}", ip);
            self.instructions[*ip] = Instruction::JumpBackward(ip - start_ins_ptr);
        });

        Ok(())
    }

    pub(crate) fn generate_break(&mut self, position: TokenPosition) -> Result<(), CodegenError> {
        if let Some(iter) = self.iterators.last_mut() {
            iter.breaks.push(self.instructions.len());
            self.instructions.push(Instruction::NoOperation);
        } else {
            return Err(CodegenError {
                error: CodegenErrorType::BreakOutsideOfLoop,
                position,
            });
        }
        Ok(())
    }

    pub(crate) fn generate_continue(&mut self, position: TokenPosition) -> Result<(), CodegenError> {
        if let Some(iter) = self.iterators.last_mut() {
            iter.continues.push(self.instructions.len());
            self.instructions.push(Instruction::NoOperation);
        } else {
            return Err(CodegenError {
                error: CodegenErrorType::ContinueOutsideOfLoop,
                position,
            });
        }
        Ok(())
    }
}