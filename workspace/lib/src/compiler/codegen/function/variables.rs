use crate::compiler::codegen::function::{Function, Variable};
use crate::compiler::codegen::syntax::{Syntax, TokenPosition};
use crate::compiler::error::{CodegenError, CodegenErrorType};
use crate::runtime::ir::instruction::Instruction;

impl Function {
    pub(crate) fn generate_parameters(&mut self, parameters: Vec<Syntax>) -> Result<(), CodegenError> {
        for param in parameters {
            if let Syntax::Variable { position, name, value } = param.clone() {
                self.generate_variable_with_value(position, name, value)?;
            } else {
                return Err(CodegenError {
                    error: CodegenErrorType::UnableToCompileParameterVariable(param),
                    position: Default::default(),
                }); // fixme add position
            };
        }

        Ok(())
    }

    pub(crate) fn generate_variable_with_value_else_default(&mut self, position: TokenPosition, name: Box<Syntax>, value: Option<Box<Syntax>>) -> Result<(), CodegenError> {

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

    pub(crate) fn generate_variable_with_value(&mut self, position: TokenPosition, name: Box<Syntax>, value: Option<Box<Syntax>>) -> Result<(), CodegenError> {

        // check if variable already exists
        if self.variables.contains_key(name.to_string().as_str()) {
            return Err(CodegenError {
                error: CodegenErrorType::VariableAlreadyDeclared(name.to_string()),
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
    pub(crate) fn generate_assignment(&mut self, position: TokenPosition, target: Box<Syntax>, value: Box<Syntax>) -> Result<(), CodegenError> {
        match *target.clone() {

            // store value in variable
            Syntax::Identifier { position, name } => {
                if self.variables.contains_key(&name) == false {
                    return Err(CodegenError {
                        error: CodegenErrorType::VariableNotDeclared(name),
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

            _ => return Err(CodegenError {
                error: CodegenErrorType::UnableToAssignItem(target),
                position,
            })
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::compiler::codegen::function::Function;
    use crate::compiler::codegen::syntax::{Syntax, TokenPosition};
    use crate::runtime::ir::instruction::Instruction;

    macro_rules! test_assignment_ok {
        ($target:expr, $value:expr, $expected:expr) => {
            {
                use crate::compiler::codegen::function::Variable;
                let mut g = Function::default();

                g.variables.insert(String::from("a"), Variable {
                    slot_index: 0,
                    name: String::from("a"),
                });
                g.variables.insert(String::from("b"), Variable {
                    slot_index: 1,
                    name: String::from("b"),
                });
                g.variables.insert(String::from("c"), Variable {
                    slot_index: 2,
                    name: String::from("c"),
                });
                g.variables.insert(String::from("rooms"), Variable {
                    slot_index: 3,
                    name: String::from("rooms"),
                });

                let _ = g.generate_assignment(TokenPosition::default(), $target, $value);
                assert_eq!(g.instructions, $expected);
            }
        };
    }

    #[test]
    fn test_identifier_assignment_with_member_access() {
        // kitchen.teacups = 2
        test_assignment_ok!(
            Box::new(Syntax::MemberAccess {
                position: TokenPosition::default(),
                target: Box::new(Syntax::Identifier {
                    position: TokenPosition::default(),
                    name: "kitchen".to_string(),
                }),
                index: Box::new(Syntax::Identifier {
                    position: TokenPosition::default(),
                    name: "teacups".to_string(),
                }),
            }),
            Box::new(Syntax::Integer(2)),
            vec![
                Instruction::LoadGlobal("kitchen".to_string()),
                Instruction::PushIdentifier("teacups".to_string()),
                Instruction::PushInteger(2),
                Instruction::SetCollectionItem,
            ]
        );
    }

    #[test]
    fn test_identifier_assignment_with_double_member_access() {
        // house.kitchen.teacups = 2
        test_assignment_ok!(
            Box::new(Syntax::MemberAccess {
                position: TokenPosition::default(),
                target: Box::new(Syntax::MemberAccess {
                    position: TokenPosition::default(),
                    target: Box::new(Syntax::Identifier {
                        position: TokenPosition::default(),
                        name: "house".to_string(),
                    }),
                    index: Box::new(Syntax::Identifier {
                        position: TokenPosition::default(),
                        name: "kitchen".to_string(),
                    }),
                }),
                index: Box::new(Syntax::Identifier {
                    position: TokenPosition::default(),
                    name: "teacups".to_string(),
                }),
            }),
            Box::new(Syntax::Integer(2)),
            vec![
                Instruction::LoadGlobal("house".to_string()),
                Instruction::LoadMember("kitchen".to_string()),
                Instruction::PushIdentifier("teacups".to_string()),
                Instruction::PushInteger(2),
                Instruction::SetCollectionItem,
            ]
        );
    }
}