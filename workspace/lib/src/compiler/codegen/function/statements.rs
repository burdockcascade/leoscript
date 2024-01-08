use log::warn;

use crate::compiler::codegen::function::Function;
use crate::compiler::codegen::syntax::{Syntax, TokenPosition};
use crate::compiler::error::CodegenError;
use crate::runtime::ir::instruction::Instruction;

impl Function {
    pub(crate) fn generate_statements(&mut self, statements: Vec<Syntax>) -> Result<(), CodegenError> {
        for statement in statements {
            self.generate_statement(Box::new(statement))?;
        }
        Ok(())
    }

    // compile a statement
    fn generate_statement(&mut self, statement: Box<Syntax>) -> Result<(), CodegenError> {
        match *statement.clone() {
            Syntax::Variable { position, name, value } => self.generate_variable_with_value_else_default(position, name, value)?,
            Syntax::Assign { position, target, value } => self.generate_assignment(position, target, value)?,
            Syntax::Call { position, .. } => self.generate_expression(position, statement.clone())?,
            Syntax::Return { position, expr } => self.generate_return(position, expr)?,
            Syntax::WhileLoop { position, condition, body } => self.generate_while_loop(position, condition, body)?,
            Syntax::ForEach { position, ident, collection, body } => self.generate_iterator(position, ident, collection, body)?,
            Syntax::IfChain { .. } => self.generate_if_else(statement)?,
            Syntax::Match { .. } => self.generate_match(statement)?,
            Syntax::Break { position } => self.generate_break(position)?,
            Syntax::Continue { position } => self.generate_continue(position)?,
            Syntax::Print { position, expr } => self.generate_print(position, expr)?,
            Syntax::Sleep { position, expr } => self.generate_sleep(position, expr)?,
            _ => warn!("unrecognized statement: {:?}", statement)
        }

        Ok(())
    }

    fn generate_print(&mut self, position: TokenPosition, expr: Box<Syntax>) -> Result<(), CodegenError> {
        self.generate_expression(position, expr)?;
        self.instructions.push(Instruction::Print);
        Ok(())
    }

    fn generate_sleep(&mut self, position: TokenPosition, expr: Box<Syntax>) -> Result<(), CodegenError> {
        self.generate_expression(position, expr)?;
        self.instructions.push(Instruction::Sleep);
        Ok(())
    }

    //==============================================================================================
    // FUNCTIONS

    // compile a return statement
    fn generate_return(&mut self, position: TokenPosition, expr: Option<Box<Syntax>>) -> Result<(), CodegenError> {
        match expr {
            Some(expr) => {
                self.generate_expression(position, expr)?;
                self.instructions.push(Instruction::Return { with_value: true });
            }
            None => {
                self.instructions.push(Instruction::Return { with_value: false });
            }
        }

        Ok(())
    }
}