use crate::compiler::codegen::function::{Function, Variable};
use crate::compiler::codegen::syntax::{Syntax, TokenPosition};
use crate::compiler::error::{CompilerError, CompilerErrorType};
use crate::runtime::ir::instruction::Instruction;

impl Function {
    pub(crate) fn generate_parameters(&mut self, parameters: Vec<Syntax>) -> Result<(), CompilerError> {
        for param in parameters {
            if let Syntax::Variable { position, name, value } = param.clone() {
                self.generate_variable_with_value(position, name, value)?;
            } else {
                return Err(CompilerError {
                    error: CompilerErrorType::UnableToCompileParameterVariable(param),
                    position: Default::default(),
                }); // fixme add position
            };
        }

        Ok(())
    }

    pub(crate) fn generate_variable_with_value_else_default(&mut self, position: TokenPosition, name: Box<Syntax>, value: Option<Box<Syntax>>) -> Result<(), CompilerError> {

        // create the variable
        self.generate_variable_with_value(position, name.clone(), value.clone())?;
        let slot_index = self.variables.get(&name.to_string()).unwrap().slot_index;

        // set default value
        if value.is_none() {
            self.instructions.push(Instruction::PushNull);
            self.instructions.push(Instruction::MoveToLocalVariable(slot_index));
        }

        Ok(())
    }

    pub(crate) fn generate_variable_with_value(&mut self, position: TokenPosition, name: Box<Syntax>, value: Option<Box<Syntax>>) -> Result<(), CompilerError> {

        // check if variable already exists
        if self.variables.contains_key(name.to_string().as_str()) {
            return Err(CompilerError {
                error: CompilerErrorType::VariableAlreadyDeclared(name.to_string()),
                position,
            });
        }

        // create the variable
        let v = Variable {
            name: name.to_string(),
            slot_index: self.variables.len(),
        };

        // add variable to list
        self.variables.insert(name.to_string(), v.clone());

        if let Some(expr) = value {
            self.generate_expression(position, expr)?;
            self.instructions.push(Instruction::MoveToLocalVariable(v.slot_index));
        }

        Ok(())
    }

    // compile assignment
    pub(crate) fn generate_assignment(&mut self, position: TokenPosition, target: Box<Syntax>, value: Box<Syntax>) -> Result<(), CompilerError> {
        match *target.clone() {

            // store value in variable
            Syntax::Identifier { position, name } => {
                if self.variables.contains_key(&name) == false {
                    return Err(CompilerError {
                        error: CompilerErrorType::VariableNotDeclared(name),
                        position,
                    });
                }

                // get the variable slot
                let slot = self.variables.get(&name).expect("variable to exist").slot_index;

                // compile the value
                self.generate_expression(position, value)?;

                // store the value
                self.instructions.push(Instruction::MoveToLocalVariable(slot));
            }

            Syntax::MemberAccess { position, target, index } => {
                self.generate_expression(position, target)?;

                match *index {
                    Syntax::Identifier { position: _position, name } => self.instructions.push(Instruction::PushIdentifier(name)),
                    _ => self.generate_expression(position, index)?
                }

                self.generate_expression(position, value)?;

                self.instructions.push(Instruction::SetCollectionItem);
            }

            Syntax::ArrayAccess { position, target, index } => {
                self.generate_expression(position, target)?;

                self.generate_expression(position, index)?;

                self.generate_expression(position, value)?;

                self.instructions.push(Instruction::SetCollectionItem);
            }

            _ => return Err(CompilerError {
                error: CompilerErrorType::UnableToAssignItem(target),
                position,
            })
        }

        Ok(())
    }
}