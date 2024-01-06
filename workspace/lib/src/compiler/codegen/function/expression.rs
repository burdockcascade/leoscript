use crate::compiler::codegen::function::Function;
use crate::compiler::codegen::syntax::{Syntax, TokenPosition};
use crate::compiler::error::{CompilerError, CompilerErrorType};
use crate::runtime::ir::instruction::Instruction;

impl Function {
    pub fn generate_expression(&mut self, position: TokenPosition, token: Box<Syntax>) -> Result<(), CompilerError> {
        match *token {
            Syntax::Null => {
                self.instructions.push(Instruction::PushNull);
            }

            Syntax::Integer(v) => {
                self.instructions.push(Instruction::PushInteger(v));
            }

            Syntax::Float(v) => {
                self.instructions.push(Instruction::PushFloat(v));
            }

            Syntax::Bool(v) => {
                self.instructions.push(Instruction::PushBool(v));
            }

            Syntax::String(v) => {
                self.instructions.push(Instruction::PushString(v));
            }

            Syntax::Identifier { name, .. } => {
                if self.variables.contains_key(&name) {
                    self.instructions.push(Instruction::LoadLocalVariable(self.variables.get(&name).unwrap().slot_index));
                } else {
                    self.instructions.push(Instruction::LoadGlobal(name));
                }
            }

            Syntax::ArrayAccess { position, target, index } => {
                self.generate_expression(position, target)?;
                self.generate_expression(position, index)?;
                self.instructions.push(Instruction::GetCollectionItem);
            }

            Syntax::MemberAccess { position, target, index } => {
                self.generate_expression(position, target)?;

                match *index {
                    Syntax::Identifier { position: _position, name } => self.instructions.push(Instruction::PushIdentifier(name)),
                    _ => self.generate_expression(position, index)?
                }

                self.instructions.push(Instruction::LoadMember);
            }

            Syntax::StaticAccess { position, target, index } => {
                self.generate_expression(position, target)?;
                match *index {
                    Syntax::Identifier { position: _position, name } => self.instructions.push(Instruction::PushIdentifier(name)),
                    _ => self.generate_expression(position, index)?
                }
                self.instructions.push(Instruction::GetCollectionItem);
            }

            Syntax::Array(elements) => {
                let array_size = elements.len();

                // Compile each element
                for element in elements {
                    self.generate_expression(position, Box::new(element))?;
                }

                // collect items into array
                self.instructions.push(Instruction::CreateCollectionAsArray(array_size));
            }

            Syntax::Dictionary(pairs) => {
                let dict_size = pairs.len();

                for (key, value) in pairs {
                    self.instructions.push(Instruction::PushString(key));
                    self.generate_expression(position, Box::from(value))?;
                }

                // collect items into dictionary
                self.instructions.push(Instruction::CreateCollectionAsDictionary(dict_size));
            }

            Syntax::AnonFunction { position, input, body } => {

                // create a new function
                let func_name = format!("lambda_{}", self.anon_functions.len());
                let f = Function::new(position, func_name.clone(), input, body);

                match f {
                    Ok(f) => { self.anon_functions.insert(func_name.clone(), f.instructions) }
                    Err(e) => return Err(e),
                };

                // push globalref onto stack
                self.instructions.push(Instruction::PushFunctionRef(func_name));
            }

            Syntax::NewObject { name, input, position } => {
                self.generate_expression(position, name)?;

                // create object
                self.instructions.push(Instruction::CreateObject);
                self.instructions.push(Instruction::PushIdentifier(String::from("constructor")));
                self.instructions.push(Instruction::LoadMember);

                let param_len = input.len() + 1;

                // compile the arguments
                for arg in input {
                    self.generate_expression(position, Box::new(arg))?;
                }

                // call the constructor
                self.instructions.push(Instruction::Call(param_len));
            }

            Syntax::Call { position, target, args } => {

                //self.generate_stack_trace_push(position.line)?;

                let mut arg_len = args.len();

                // push function name onto stack
                self.generate_expression(position, target.clone())?;

                match *target {
                    Syntax::MemberAccess { .. } => arg_len += 1,
                    _ => {}
                }

                // compile the arguments
                for arg in args {
                    self.generate_expression(position, Box::new(arg))?;
                }

                self.instructions.push(Instruction::Call(arg_len));

                //self.generate_stack_trace_pop()?;
            }

            Syntax::Eq { expr1, expr2 } => {
                self.generate_expression(position, expr1)?;
                self.generate_expression(position, expr2)?;
                self.instructions.push(Instruction::Equal);
            }

            Syntax::Ne { expr1, expr2 } => {
                self.generate_expression(position, expr1)?;
                self.generate_expression(position, expr2)?;
                self.instructions.push(Instruction::NotEqual);
            }

            Syntax::Add { expr1, expr2 } => {
                self.generate_expression(position, expr1)?;
                self.generate_expression(position, expr2)?;
                self.instructions.push(Instruction::Add);
            }

            Syntax::Sub { expr1, expr2 } => {
                self.generate_expression(position, expr1)?;
                self.generate_expression(position, expr2)?;
                self.instructions.push(Instruction::Sub);
            }

            Syntax::Mul { expr1, expr2 } => {
                self.generate_expression(position, expr1)?;
                self.generate_expression(position, expr2)?;
                self.instructions.push(Instruction::Multiply);
            }

            Syntax::Div { expr1, expr2 } => {
                self.generate_expression(position, expr1)?;
                self.generate_expression(position, expr2)?;
                self.instructions.push(Instruction::Divide);
            }

            Syntax::Pow { expr1, expr2 } => {
                self.generate_expression(position, expr1)?;
                self.generate_expression(position, expr2)?;
                self.instructions.push(Instruction::Pow);
            }

            Syntax::Lt { expr1, expr2 } => {
                self.generate_expression(position, expr1)?;
                self.generate_expression(position, expr2)?;
                self.instructions.push(Instruction::LessThan);
            }

            Syntax::Le { expr1, expr2 } => {
                self.generate_expression(position, expr1)?;
                self.generate_expression(position, expr2)?;
                self.instructions.push(Instruction::LessThanOrEqual);
            }

            Syntax::Gt { expr1, expr2 } => {
                self.generate_expression(position, expr1)?;
                self.generate_expression(position, expr2)?;
                self.instructions.push(Instruction::GreaterThan);
            }

            Syntax::Ge { expr1, expr2 } => {
                self.generate_expression(position, expr1)?;
                self.generate_expression(position, expr2)?;
                self.instructions.push(Instruction::GreaterThanOrEqual);
            }

            Syntax::Not { expr } => {
                self.generate_expression(position, expr)?;
                self.instructions.push(Instruction::Not);
            }

            Syntax::And { expr1, expr2 } => {
                self.generate_expression(position, expr1)?;
                self.generate_expression(position, expr2)?;
                self.instructions.push(Instruction::And);
            }

            Syntax::Or { expr1, expr2 } => {
                self.generate_expression(position, expr1)?;
                self.generate_expression(position, expr2)?;
                self.instructions.push(Instruction::Or);
            }

            Syntax::Range { .. } => unimplemented!("range not implemented"),

            // handle unreadable token and print what it is
            _ => return Err(CompilerError {
                error: CompilerErrorType::InvalidExpressionItem(token),
                position,
            })
        }

        Ok(())
    }
}

mod test {
    use std::collections::HashMap;

    use crate::compiler::codegen::function::Function;
    use crate::compiler::codegen::function::Variable;
    use crate::compiler::codegen::syntax::Syntax;
    use crate::runtime::ir::instruction::Instruction;
    use crate::runtime::ir::variant::ValueType;

    macro_rules! test_expression_ok {
        ($token:expr, $expected:expr) => {
            let position = Default::default();

            let mut variables = HashMap::new();
            variables.insert(String::from("a"), Variable {
                slot_index: 0,
                name: String::from("a"),
                as_type: ValueType::Any
            });
            variables.insert(String::from("b"), Variable {
                slot_index: 1,
                name: String::from("b"),
                as_type: ValueType::Any
            });
            variables.insert(String::from("c"), Variable {
                slot_index: 2,
                name: String::from("c"),
                as_type: ValueType::Any
            });
            variables.insert(String::from("d"), Variable {
                slot_index: 3,
                name: String::from("d"),
                as_type: ValueType::Any
            });

            let mut generator = Function::default();
            generator.variables = variables;
            generator.generate_expression(position, Box::from($token)).unwrap();

            assert_eq!(generator.instructions, $expected);
        };
    }

    // basic expressions

    #[test]
    fn expression_null() {
        test_expression_ok!(
            Syntax::Null,
            vec![Instruction::PushNull]
        );
    }

    #[test]
    fn expression_integer() {
        test_expression_ok!(
            Syntax::Integer(1),
            vec![Instruction::PushInteger(1)]
        );
    }

    #[test]
    fn expression_float() {
        test_expression_ok!(
            Syntax::Float(1.0),
            vec![Instruction::PushFloat(1.0)]
        );
    }

    #[test]
    fn expression_bool() {
        test_expression_ok!(
            Syntax::Bool(true),
            vec![Instruction::PushBool(true)]
        );
    }

    #[test]
    fn expression_string() {
        test_expression_ok!(
            Syntax::String(String::from("hello")),
            vec![Instruction::PushString(String::from("hello"))]
        );
    }

    // math expressions

    #[test]
    fn add_numbers() {
        test_expression_ok!(
            Syntax::Add {
                expr1: Box::new(Syntax::Integer(1)),
                expr2: Box::new(Syntax::Integer(2)),
            },
            vec![
                Instruction::PushInteger(1),
                Instruction::PushInteger(2),
                Instruction::Add,
            ]
        );
    }

    #[test]
    fn subtract_numbers() {
        test_expression_ok!(
            Syntax::Sub {
                expr1: Box::new(Syntax::Integer(1)),
                expr2: Box::new(Syntax::Integer(2)),
            },
            vec![
                Instruction::PushInteger(1),
                Instruction::PushInteger(2),
                Instruction::Sub,
            ]
        );
    }

    #[test]
    fn multiply_numbers() {
        test_expression_ok!(
            Syntax::Mul {
                expr1: Box::new(Syntax::Integer(1)),
                expr2: Box::new(Syntax::Integer(2)),
            },
            vec![
                Instruction::PushInteger(1),
                Instruction::PushInteger(2),
                Instruction::Multiply,
            ]
        );
    }

    #[test]
    fn divide_numbers() {
        test_expression_ok!(
            Syntax::Div {
                expr1: Box::new(Syntax::Integer(1)),
                expr2: Box::new(Syntax::Integer(2)),
            },
            vec![
                Instruction::PushInteger(1),
                Instruction::PushInteger(2),
                Instruction::Divide,
            ]
        );
    }

    #[test]
    fn power_numbers() {
        test_expression_ok!(
            Syntax::Pow {
                expr1: Box::new(Syntax::Integer(1)),
                expr2: Box::new(Syntax::Integer(2)),
            },
            vec![
                Instruction::PushInteger(1),
                Instruction::PushInteger(2),
                Instruction::Pow,
            ]
        );
    }

    // boolean expressions

    #[test]
    fn less_than_numbers() {
        test_expression_ok!(
            Syntax::Lt {
                expr1: Box::new(Syntax::Integer(1)),
                expr2: Box::new(Syntax::Integer(2)),
            },
            vec![
                Instruction::PushInteger(1),
                Instruction::PushInteger(2),
                Instruction::LessThan,
            ]
        );
    }

    #[test]
    fn less_than_or_equal_numbers() {
        test_expression_ok!(
            Syntax::Le {
                expr1: Box::new(Syntax::Integer(1)),
                expr2: Box::new(Syntax::Integer(2)),
            },
            vec![
                Instruction::PushInteger(1),
                Instruction::PushInteger(2),
                Instruction::LessThanOrEqual,
            ]
        );
    }

    #[test]
    fn greater_than_numbers() {
        test_expression_ok!(
            Syntax::Gt {
                expr1: Box::new(Syntax::Integer(1)),
                expr2: Box::new(Syntax::Integer(2)),
            },
            vec![
                Instruction::PushInteger(1),
                Instruction::PushInteger(2),
                Instruction::GreaterThan,
            ]
        );
    }

    #[test]
    fn greater_than_or_equal_numbers() {
        test_expression_ok!(
            Syntax::Ge {
                expr1: Box::new(Syntax::Integer(1)),
                expr2: Box::new(Syntax::Integer(2)),
            },
            vec![
                Instruction::PushInteger(1),
                Instruction::PushInteger(2),
                Instruction::GreaterThanOrEqual,
            ]
        );
    }

    #[test]
    fn equal_numbers() {
        test_expression_ok!(
            Syntax::Eq {
                expr1: Box::new(Syntax::Integer(1)),
                expr2: Box::new(Syntax::Integer(2)),
            },
            vec![
                Instruction::PushInteger(1),
                Instruction::PushInteger(2),
                Instruction::Equal,
            ]
        );
    }

    #[test]
    fn long_math() {
        // 1 + 2 * 3
        test_expression_ok!(
            Syntax::Add {
                expr1: Box::new(Syntax::Integer(1)),
                expr2: Box::new(Syntax::Mul {
                    expr1: Box::new(Syntax::Integer(2)),
                    expr2: Box::new(Syntax::Integer(3)),
                }),
            },
            vec![
                Instruction::PushInteger(1),
                Instruction::PushInteger(2),
                Instruction::PushInteger(3),
                Instruction::Multiply,
                Instruction::Add,
            ]
        );
    }

    // new object expressions

    #[test]
    fn new_object_no_variables() {
        // new myservice()
        test_expression_ok!(
            Syntax::NewObject {
                name: Box::new(Syntax::Identifier {
                    name: String::from("myservice"),
                    position: Default::default()
                }),
                input: vec![],
                position: Default::default(),
            },
            vec![
                Instruction::LoadGlobal(String::from("myservice")),
                Instruction::CreateObject,
                Instruction::PushIdentifier(String::from("constructor")),
                Instruction::LoadMember,
                Instruction::Call(1),
            ]
        );
    }

    #[test]
    fn new_object_with_variables() {
        // new myservice(1, a)
        test_expression_ok!(
            Syntax::NewObject {
                name: Box::new(Syntax::Identifier {
                    name: String::from("myservice"),
                    position: Default::default()
                }),
                input: vec![
                    Syntax::Integer(1),
                    Syntax::Identifier {
                        name: String::from("a"),
                        position: Default::default()
                    },
                ],
                position: Default::default(),
            },
            vec![
                Instruction::LoadGlobal(String::from("myservice")),
                Instruction::CreateObject,
                Instruction::PushIdentifier(String::from("constructor")),
                Instruction::LoadMember,
                Instruction::PushInteger(1),
                Instruction::LoadLocalVariable(0),
                Instruction::Call(3),
            ]
        );
    }

    #[test]
    fn new_object_with_arguments_from_module() {
        // new mymod::myservice(1, 2)
        test_expression_ok!(
            Syntax::NewObject {
                name: Box::new(Syntax::StaticAccess {
                    position: Default::default(),
                    target: Box::new(Syntax::Identifier {
                        name: String::from("mymod"),
                        position: Default::default()
                    }),
                    index: Box::new(Syntax::Identifier {
                        name: String::from("myservice"),
                        position: Default::default()
                    }),
                }),
                input: vec![
                    Syntax::Integer(1),
                    Syntax::Integer(2),
                ],
                position: Default::default(),
            },
            vec![
                Instruction::LoadGlobal(String::from("mymod")),
                Instruction::PushIdentifier(String::from("myservice")),
                Instruction::GetCollectionItem,
                Instruction::CreateObject,
                Instruction::PushIdentifier(String::from("constructor")),
                Instruction::LoadMember,
                Instruction::PushInteger(1),
                Instruction::PushInteger(2),
                Instruction::Call(3),
            ]
        );
    }

    // call expressions

    #[test]
    fn call_function_with_arguments() {
        // myfunc(1, 2)
        test_expression_ok!(
            Syntax::Call {
                position: Default::default(),
                target: Box::new(Syntax::Identifier {
                    name: String::from("myfunc"),
                    position: Default::default()
                }),
                args: vec![Syntax::Integer(1), Syntax::Integer(2)],
            },
            vec![
                Instruction::LoadGlobal(String::from("myfunc")),
                Instruction::PushInteger(1),
                Instruction::PushInteger(2),
                Instruction::Call(2),
            ]
        );
    }

    #[test]
    fn call_function_with_expression() {
        // myfunc(1 + 2)
        test_expression_ok!(
            Syntax::Call {
                position: Default::default(),
                target: Box::new(Syntax::Identifier {
                    name: String::from("myfunc"),
                    position: Default::default()
                }),
                args: vec![Syntax::Add {
                    expr1: Box::new(Syntax::Integer(1)),
                    expr2: Box::new(Syntax::Integer(2)),
                }],
            },
            vec![
                Instruction::LoadGlobal(String::from("myfunc")),
                Instruction::PushInteger(1),
                Instruction::PushInteger(2),
                Instruction::Add,
                Instruction::Call(1),
            ]
        );
    }

    #[test]
    fn call_function_on_object() {
        // myservice.myfunc(1, 2)
        test_expression_ok!(
            Syntax::Call {
                position: Default::default(),
                target: Box::new(Syntax::MemberAccess {
                    position: Default::default(),
                    target: Box::new(Syntax::Identifier {
                        name: String::from("myservice"),
                        position: Default::default()
                    }),
                    index: Box::new(Syntax::Identifier {
                        name: String::from("myfunc"),
                        position: Default::default()
                    }),
                }),
                args: vec![Syntax::Integer(1), Syntax::Integer(2)],
            },
            vec![
                Instruction::LoadGlobal(String::from("myservice")),
                Instruction::PushIdentifier(String::from("myfunc")),
                Instruction::LoadMember,
                Instruction::PushInteger(1),
                Instruction::PushInteger(2),
                Instruction::Call(3),
            ]
        );
    }

    #[test]
    fn call_function_on_new_object_from_module() {
        // new mymod::myservice(1, 2).print()
        test_expression_ok!(
            Syntax::Call {
                position: Default::default(),
                target: Box::new(Syntax::MemberAccess {
                    position: Default::default(),
                    target: Box::new(Syntax::NewObject {
                        name: Box::new(Syntax::StaticAccess {
                            position: Default::default(),
                            target: Box::new(Syntax::Identifier {
                                name: String::from("mymod"),
                                position: Default::default()
                            }),
                            index: Box::new(Syntax::Identifier {
                                name: String::from("myservice"),
                                position: Default::default()
                            }),
                        }),
                        input: vec![
                            Syntax::Integer(1),
                            Syntax::Integer(2),
                        ],
                        position: Default::default(),
                    }),
                    index: Box::new(Syntax::Identifier {
                        name: String::from("print"),
                        position: Default::default()
                    }),
                }),
                args: vec![],
            },
            vec![
                Instruction::LoadGlobal(String::from("mymod")),
                Instruction::PushIdentifier(String::from("myservice")),
                Instruction::GetCollectionItem,
                Instruction::CreateObject,
                Instruction::PushIdentifier(String::from("constructor")),
                Instruction::LoadMember,
                Instruction::PushInteger(1),
                Instruction::PushInteger(2),
                Instruction::Call(3),
                Instruction::PushIdentifier(String::from("print")),
                Instruction::LoadMember,
                Instruction::Call(1),
            ]
        );
    }
}