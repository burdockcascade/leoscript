use crate::compiler::codegen::function::Function;
use crate::compiler::codegen::syntax::{Syntax, TokenPosition};
use crate::compiler::error::{CodegenError, CodegenErrorType};
use crate::runtime::ir::instruction::Instruction;

impl Function {
    pub(crate) fn generate_if_else(&mut self, syntax: Box<Syntax>) -> Result<(), CodegenError> {
        let Syntax::IfChain { position: _, arms } = *syntax else {
            return Err(CodegenError {
                error: CodegenErrorType::IfStatementInvalid,
                position: TokenPosition::default(),
            });
        };

        let mut jump_to_end = vec![];

        for if_statement in arms {
            match if_statement {
                Syntax::If { position, condition, body } => {

                    // Compile If Statement
                    self.generate_expression(position, condition)?;

                    // jump to next condition if false
                    let jump_to_next = self.instructions.len();
                    self.instructions.push(Instruction::Halt(String::from("unknown next condition to jump to")));

                    // Compile statements
                    self.generate_statements(body)?;

                    // jump to end
                    jump_to_end.push(self.instructions.len());
                    self.instructions.push(Instruction::Halt(String::from("can not jump to end")));

                    // Update Jump to next condition
                    self.instructions[jump_to_next] = Instruction::JumpForwardIfFalse(self.instructions.len() - jump_to_next);
                }
                Syntax::Else { body, .. } => {
                    self.generate_statements(body)?;
                }
                _ => return Err(CodegenError {
                    error: CodegenErrorType::IfStatementInvalid,
                    position: TokenPosition::default(),
                })
            }
        }

        // Update End Jumps
        for jump in jump_to_end {
            self.instructions[jump] = Instruction::JumpForward(self.instructions.len() - jump);
        }

        Ok(())
    }

    //==============================================================================================
    // MATCH

    pub(crate) fn generate_match(&mut self, syntax: Box<Syntax>) -> Result<(), CodegenError> {
        let mut jump_to_end = vec![];

        let Syntax::Match { position, expr, arms, default } = *syntax else {
            return Err(CodegenError {
                error: CodegenErrorType::InvalidMatch,
                position: Default::default(),
            });
        };

        for arm in arms {
            match arm {
                Syntax::Case { position, condition, body } => {

                    // Compile Expression
                    self.generate_expression(position, expr.clone())?;

                    // Compile If Statement
                    self.generate_expression(position, condition)?;

                    // Compare
                    self.instructions.push(Instruction::Equal);

                    // jump to next condition if false
                    let jump_to_next = self.instructions.len();
                    self.instructions.push(Instruction::Halt(String::from("unknown next condition to jump to")));

                    // Compile statements
                    self.generate_statements(body)?;

                    // jump to end
                    jump_to_end.push(self.instructions.len());
                    self.instructions.push(Instruction::Halt(String::from("can not jump to end")));

                    // Update Jump to next condition
                    self.instructions[jump_to_next] = Instruction::JumpForwardIfFalse(self.instructions.len() - jump_to_next);
                }
                _ => return Err(CodegenError {
                    error: CodegenErrorType::InvalidMatchArm,
                    position,
                })
            }
        }

        // if default exists then execute it
        if let Some(def) = default {
            match *def {
                Syntax::DefaultCase { body, .. } => {
                    self.generate_statements(body)?;
                }
                _ => return Err(CodegenError {
                    error: CodegenErrorType::InvalidDefaultCase,
                    position: Default::default(),
                })
            }
        }

        // Update End Jumps
        for jump in jump_to_end {
            self.instructions[jump] = Instruction::JumpForward(self.instructions.len() - jump);
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::compiler::codegen::function::Function;
    use crate::compiler::codegen::syntax::Syntax;
    use crate::runtime::ir::instruction::Instruction;

    #[test]
    fn match_with_default() {
        let syntax = Syntax::Match {
            position: Default::default(),
            expr: Box::new(Syntax::Integer(1)),
            arms: vec![
                Syntax::Case {
                    position: Default::default(),
                    condition: Box::new(Syntax::Integer(1)),
                    body: vec![Syntax::Print {
                        position: Default::default(),
                        expr: Box::new(Syntax::String("one".to_string())),
                    }],
                },
                Syntax::Case {
                    position: Default::default(),
                    condition: Box::new(Syntax::Integer(2)),
                    body: vec![Syntax::Print {
                        position: Default::default(),
                        expr: Box::new(Syntax::String("two".to_string())),
                    }],
                },
            ],
            default: Some(Box::from(Syntax::DefaultCase {
                position: Default::default(),
                body: vec![Syntax::Print {
                    position: Default::default(),
                    expr: Box::new(Syntax::String("default".to_string())),
                }],
            })),
        };

        let mut generator = Function::default();
        generator.generate_match(Box::from(syntax)).unwrap();

        assert_eq!(generator.instructions, vec![
            Instruction::PushInteger(1),
            Instruction::PushInteger(1),
            Instruction::Equal,
            Instruction::JumpForwardIfFalse(4),
            Instruction::PushString("one".to_string()),
            Instruction::Print,
            Instruction::JumpForward(10),
            Instruction::PushInteger(1),
            Instruction::PushInteger(2),
            Instruction::Equal,
            Instruction::JumpForwardIfFalse(4),
            Instruction::PushString("two".to_string()),
            Instruction::Print,
            Instruction::JumpForward(3),
            Instruction::PushString("default".to_string()),
            Instruction::Print,
        ]);
    }

    #[test]
    fn single_if() {
        let syntax = Syntax::IfChain {
            position: Default::default(),
            arms: vec![
                Syntax::If {
                    position: Default::default(),
                    condition: Box::new(Syntax::Integer(1)),
                    body: vec![Syntax::Print {
                        position: Default::default(),
                        expr: Box::new(Syntax::String("hello".to_string())),
                    }],
                }
            ],

        };

        let mut generator = Function::default();
        generator.generate_if_else(Box::from(syntax)).unwrap();

        assert_eq!(generator.instructions, vec![
            Instruction::PushInteger(1),
            Instruction::JumpForwardIfFalse(4),
            Instruction::PushString("hello".to_string()),
            Instruction::Print,
            Instruction::JumpForward(1),
        ]);
    }

    #[test]
    fn if_else() {
        let syntax = Syntax::IfChain {
            position: Default::default(),
            arms: vec![
                Syntax::If {
                    position: Default::default(),
                    condition: Box::new(Syntax::Integer(1)),
                    body: vec![Syntax::Print {
                        position: Default::default(),
                        expr: Box::new(Syntax::String("hello".to_string())),
                    }],
                },
                Syntax::Else {
                    position: Default::default(),
                    body: vec![Syntax::Print {
                        position: Default::default(),
                        expr: Box::new(Syntax::String("world".to_string())),
                    }],
                },
            ],

        };

        let mut generator = Function::default();
        generator.generate_if_else(Box::from(syntax)).unwrap();

        assert_eq!(generator.instructions, vec![
            Instruction::PushInteger(1),
            Instruction::JumpForwardIfFalse(4),
            Instruction::PushString("hello".to_string()),
            Instruction::Print,
            Instruction::JumpForward(3),
            Instruction::PushString("world".to_string()),
            Instruction::Print,
        ]);
    }
}