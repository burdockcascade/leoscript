use std::collections::HashMap;

use log::{trace, warn};

use crate::compiler::codegen::variable::Variable;
use crate::compiler::codegen::syntax::{Syntax, TokenPosition};
use crate::compiler::error::{CompilerError, CompilerErrorType};
use crate::runtime::ir::instruction::Instruction;
use crate::runtime::ir::stacktrace::StackTrace;
use crate::runtime::ir::variant::ValueType;

#[derive(Clone)]
pub struct Function {
    name: String,
    pub instructions: Vec<Instruction>,
    pub variables: HashMap<String, Variable>,
    pub anon_functions: HashMap<String, Vec<Instruction>>,
    iterators: Vec<IteratorTracker>,
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
            anon_functions: Default::default(),
            iterators: vec![],
        };

        f.generate_stack_trace_push(position.line)?;

        // store the parameters as variables
        f.generate_parameters(parameters.clone())?;

        // compile the statements
        f.generate_statements(body)?;

        // if tha last instruction is not a return then add one
        if matches!(f.instructions.last(), Some(Instruction::ReturnWithValue)) == false && matches!(f.instructions.last(), Some(Instruction::Return)) == false {
            f.instructions.push(Instruction::Return);
        }

        f.generate_stack_trace_pop()?;

        f.instructions.insert(0, Instruction::SetVariableBuffer(f.variables.len()));

        Ok(f)
    }

    fn generate_stack_trace_push(&mut self, line: usize) -> Result<(), CompilerError> {
        self.instructions.push(Instruction::PushStackTrace(StackTrace {
            line,
            file: String::from("unknown"),
            function: self.name.clone(),
        }));

        Ok(())
    }

    fn generate_stack_trace_pop(&mut self) -> Result<(), CompilerError> {
        self.instructions.push(Instruction::PopStackTrace);
        Ok(())
    }

    //==============================================================================================
    // STATEMENTS

    // compile a list of statements
    fn generate_statements(&mut self, statements: Vec<Syntax>) -> Result<(), CompilerError> {
        for statement in statements {
            self.generate_statement(Box::new(statement))?;
        }
        Ok(())
    }

    // compile a statement
    fn generate_statement(&mut self, statement: Box<Syntax>) -> Result<(), CompilerError> {
        match *statement {
            Syntax::Variable { position, name, as_type, value } => self.generate_variable_with_value_else_default(position, name, as_type, value)?,
            Syntax::Assign { position, ident, value } => self.generate_assignment(position, ident, value)?,
            Syntax::Call { position, name, input } => self.generate_call(position, name, input)?,
            Syntax::Return { position, expr } => self.generate_return(position, expr)?,
            Syntax::WhileLoop { position, condition, body } => self.generate_while_loop(position, condition, body)?,
            Syntax::ForI { position, ident, start, step, end, body } => self.generate_iterator(position, ident, start, step, end, body)?,
            Syntax::ForEach { position, ident, collection, body } => self.generate_iterator(position, ident, Box::new(Syntax::Integer(0)), Box::new(Syntax::Integer(1)), collection, body)?,
            Syntax::IfChain { position, chain } => self.generate_if_else(position, chain)?,
            Syntax::Match { position, expr, arms, default } => self.generate_match(position, expr, arms, default)?,
            Syntax::Comment { .. } => {}
            Syntax::DotChain { position, start, chain } => self.generate_chain(position, start, chain)?,
            Syntax::Break { position } => self.generate_break(position)?,
            Syntax::Continue { position } => self.generate_continue(position)?,
            Syntax::Print { position, expr } => self.generate_print(position, expr)?,
            Syntax::Sleep { position, expr } => self.generate_sleep(position, expr)?,
            _ => warn!("unrecognized statement: {:?}", statement)
        }

        Ok(())
    }

    fn generate_print(&mut self, position: TokenPosition, expr: Box<Syntax>) -> Result<(), CompilerError> {
        self.generate_expression(position, expr)?;
        self.instructions.push(Instruction::Print);
        Ok(())
    }

    fn generate_sleep(&mut self, position: TokenPosition, expr: Box<Syntax>) -> Result<(), CompilerError> {
        self.generate_expression(position, expr)?;
        self.instructions.push(Instruction::Sleep);
        Ok(())
    }

    //==============================================================================================
    // VARIABLES

    fn generate_parameters(&mut self, parameters: Vec<Syntax>) -> Result<(), CompilerError> {
        trace!("add_parameters({:?})", parameters);

        for param in parameters {
            if let Syntax::Variable { position, name, as_type, value } = param.clone() {
                self.generate_variable_with_value(position, name, as_type, value)?;
            } else {
                return Err(CompilerError {
                    error: CompilerErrorType::UnableToCompileParameterVariable(param),
                    position: Default::default()
                }); // fixme add position
            };
        }

        Ok(())
    }

    fn generate_variable_with_value_else_default(&mut self, position: TokenPosition, name: String, as_type: Option<String>, value: Option<Box<Syntax>>) -> Result<(), CompilerError> {
        trace!("generate_variable_with_value_else_default({:?}, {:?}, {:?}, {:?})", position, name, as_type, value);

        // create the variable
        self.generate_variable_with_value(position, name.clone(), as_type, value.clone())?;
        let slot_index = self.variables.get(&name).unwrap().slot_index;

        // set default value
        if value.is_none() {
            self.instructions.push(Instruction::PushNull);
            self.instructions.push(Instruction::MoveToLocalVariable(slot_index));
        }

        Ok(())
    }

    fn generate_variable_with_value(&mut self, position: TokenPosition, name: String, as_type: Option<String>, value: Option<Box<Syntax>>) -> Result<(), CompilerError> {
        trace!("generate_variable_with_value({:?}, {:?}, {:?}, {:?})", position, name, as_type, value);

        // check if variable already exists
        if self.variables.contains_key(name.as_str()) {
            return Err(CompilerError {
                error: CompilerErrorType::VariableAlreadyDeclared(name),
                position
            });
        }

        // create the variable
        let v = Variable {
            name: name.clone(),
            as_type: match as_type {
                Some(t) => {
                    match t.to_lowercase().as_str() {
                        "integer" => ValueType::Integer,
                        "float" => ValueType::Float,
                        "string" => ValueType::String,
                        "boolean" => ValueType::Bool,
                        "array" => ValueType::Array,
                        "dictionary" => ValueType::Dictionary,
                        _ => ValueType::Global(t)
                    }
                }
                None => {
                    ValueType::Any
                }
            },
            slot_index: self.variables.len(),
        };

        // add variable to list
        self.variables.insert(name.clone(), v.clone());

        if let Some(expr) = value {
            self.generate_expression(position, expr)?;
            self.instructions.push(Instruction::MoveToLocalVariable(v.slot_index));
        }

        Ok(())
    }

    // compile assignment
    fn generate_assignment(&mut self, position: TokenPosition, left: Box<Syntax>, right: Box<Syntax>) -> Result<(), CompilerError> {
        trace!("generate_assignment({:?}, {:?}, {:?})", position, left, right);

        match *left.clone() {

            // store value in variable
            Syntax::Identifier { position, name } => {
                if self.variables.contains_key(&name) == false {
                    return Err(CompilerError {
                        error: CompilerErrorType::VariableNotDeclared(name),
                        position
                    });
                }

                // get the variable slot
                let slot = self.variables.get(&name).expect("variable to exist").slot_index;

                // compile the value
                self.generate_expression(position, right)?;

                // store the value
                self.instructions.push(Instruction::MoveToLocalVariable(slot));
            }

            Syntax::DotChain { position, start, mut chain } => {

                // remove last item from chain
                let last_item = chain.pop().expect("chain to have at least one item");

                self.generate_chain(position, start, chain)?;
                self.generate_expression(position, right)?;

                match last_item {
                    Syntax::Identifier { name, .. } => {
                        self.instructions.push(Instruction::PushString(name.to_string()));
                        self.instructions.push(Instruction::SetCollectionItem);
                    }
                    Syntax::CollectionIndex(index) => {
                        self.generate_expression(position, index)?;
                        self.instructions.push(Instruction::SetCollectionItem);
                    }
                    _ => return Err(CompilerError {
                        error: CompilerErrorType::UnableToCompileChainItem(last_item),
                        position
                    })
                }
            }

            _ => return Err(CompilerError {
                error: CompilerErrorType::UnableToAssignItem(left),
                position
            })
        }

        Ok(())
    }

    //==============================================================================================
    // FUNCTIONS

    // compile a function call
    fn generate_call(&mut self, position: TokenPosition, name: Box<Syntax>, args: Vec<Syntax>) -> Result<(), CompilerError> {
        self.generate_stack_trace_push(position.line)?;

        let arg_len = args.len();
        let function_name = name.to_string();

        if self.variables.contains_key(&function_name) {
            self.instructions.push(Instruction::LoadLocalVariable(self.variables.get(&function_name).unwrap().slot_index));
        } else {
            self.instructions.push(Instruction::PushFunctionRef(function_name));
        }

        // compile the arguments
        for arg in args {
            self.generate_expression(position, Box::new(arg))?;
        }

        self.instructions.push(Instruction::Call(arg_len));

        self.generate_stack_trace_pop()?;

        Ok(())
    }

    // compile a return statement
    fn generate_return(&mut self, position: TokenPosition, expr: Option<Box<Syntax>>) -> Result<(), CompilerError> {
        match expr {
            Some(expr) => {
                self.generate_expression(position, expr)?;
                self.instructions.push(Instruction::ReturnWithValue);
            }
            None => {
                self.instructions.push(Instruction::Return);
            }
        }

        Ok(())
    }


    //==============================================================================================
    // CLASSES

    fn generate_new_object(&mut self, position: TokenPosition, ident: Box<Syntax>, params: Vec<Syntax>) -> Result<(), CompilerError> {
        let (start, chain) = match *ident {
            Syntax::DotChain { start, chain, .. } => (start, chain),
            _ => return Err(CompilerError {
                error: CompilerErrorType::UnableToCreateNewObjectFrom(ident),
                position
            })
        };

        // compile the chain
        self.generate_chain(position, start, chain)?;

        // create object
        self.instructions.push(Instruction::CreateObject);
        self.instructions.push(Instruction::LoadMember(String::from("constructor")));

        let param_len = params.len() + 1;

        // compile the arguments
        for arg in params {
            self.generate_expression(position, Box::new(arg))?;
        }

        // call the constructor
        self.instructions.push(Instruction::Call(param_len));


        Ok(())
    }

    //==============================================================================================
    // IF

    // compile if statement
    fn generate_if_else(&mut self, _position: TokenPosition, ifs: Vec<Syntax>) -> Result<(), CompilerError> {
        let mut jump_to_end = vec![];

        for if_statement in ifs {
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
                _ => return Err(CompilerError {
                    error: CompilerErrorType::IfStatementInvalid,
                    position: TokenPosition::default()
                })
            }
        }

        // Update End Jumps
        for jump in jump_to_end {
            trace!("updating jump to end: {}", jump);
            self.instructions[jump] = Instruction::JumpForward(self.instructions.len() - jump);
        }

        Ok(())
    }

    //==============================================================================================
    // MATCH

    fn generate_match(&mut self, position: TokenPosition, expr: Box<Syntax>, arms: Vec<Syntax>, default: Option<Box<Syntax>>) -> Result<(), CompilerError> {
        let mut jump_to_end = vec![];

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
                _ => return Err(CompilerError {
                    error: CompilerErrorType::InvalidMatchArm,
                    position
                })
            }
        }

        // if default exists then execute it
        if let Some(def) = default {
            match *def {
                Syntax::DefaultCase { body, .. } => {
                    self.generate_statements(body)?;
                }
                _ => return Err(CompilerError {
                    error: CompilerErrorType::InvalidDefaultCase,
                    position: Default::default()
                })
            }
        }

        // Update End Jumps
        for jump in jump_to_end {
            trace!("updating jump to end: {}", jump);
            self.instructions[jump] = Instruction::JumpForward(self.instructions.len() - jump);
        }

        Ok(())
    }

    //==============================================================================================
    // LOOPS

    fn generate_iterator(&mut self, position: TokenPosition, var: Box<Syntax>, counter_start_at: Box<Syntax>, counter_step: Box<Syntax>, target: Box<Syntax>, block: Vec<Syntax>) -> Result<(), CompilerError> {
        self.iterators.push(IteratorTracker {
            breaks: vec![],
            continues: vec![],
        });

        // fixme this is parse error not a compile error
        if let Syntax::Identifier { position, name } = *var.clone() {
            self.generate_variable_with_value(position, name, Some(String::from("Integer")), None)?;
        } else {
            return Err(CompilerError {
                error: CompilerErrorType::UnableToIterateOver(var),
                position
            });
        };

        // get variable slot
        let var_slot = self.variables.get(var.to_string().as_str()).unwrap().slot_index;

        // compile target
        self.generate_expression(position, target)?;

        // compile counter step
        self.generate_expression(position, counter_step)?;

        // compile counter start
        self.generate_expression(position, counter_start_at)?;

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
    fn generate_while_loop(&mut self, position: TokenPosition, expr: Box<Syntax>, block: Vec<Syntax>) -> Result<(), CompilerError> {
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

    fn update_iterator_jumps(&mut self, position: TokenPosition, start_ins_ptr: usize) -> Result<(), CompilerError> {
        // get iterator
        let Some(it) = self.iterators.pop() else {
            return Err(CompilerError {
                error: CompilerErrorType::NoIteratorJumpsFound,
                position
            })
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

    fn generate_break(&mut self, position: TokenPosition) -> Result<(), CompilerError> {
        if let Some(iter) = self.iterators.last_mut() {
            trace!("adding break to: {}", self.instructions.len());
            iter.breaks.push(self.instructions.len());
            self.instructions.push(Instruction::NoOperation);
        } else {
            return Err(CompilerError {
                error: CompilerErrorType::BreakOutsideOfLoop,
                position
            })
        }
        Ok(())
    }

    fn generate_continue(&mut self, position: TokenPosition) -> Result<(), CompilerError> {
        if let Some(iter) = self.iterators.last_mut() {
            trace!("adding continue to: {}", self.instructions.len());
            iter.continues.push(self.instructions.len());
            self.instructions.push(Instruction::NoOperation);
        } else {
            return Err(CompilerError {
                error: CompilerErrorType::ContinueOutsideOfLoop,
                position
            })
        }
        Ok(())
    }

    //==============================================================================================
    // DOT CHAIN

    fn generate_chain(&mut self, position: TokenPosition, start: Box<Syntax>, chain: Vec<Syntax>) -> Result<(), CompilerError> {

        // load the start of the chain
        self.generate_expression(position, start)?;

        // for each item in chain
        for item in chain {

            // push load object member instruction onto stack
            match item {
                Syntax::Identifier { name, .. } => {
                    self.instructions.push(Instruction::PushString(name.to_string()));
                    self.instructions.push(Instruction::GetCollectionItem);
                }
                Syntax::CollectionIndex(index) => {
                    self.generate_expression(position, index)?;
                    self.instructions.push(Instruction::GetCollectionItem);
                }
                Syntax::Call { name, input, .. } => {

                    // load method
                    self.instructions.push(Instruction::LoadMember(name.to_string()));

                    let arg_len = input.len() + 1;

                    // compile the arguments
                    for arg in input {
                        self.generate_expression(position, Box::new(arg))?;
                    }

                    self.instructions.push(Instruction::Call(arg_len));
                }
                _ => return Err(CompilerError {
                    error: CompilerErrorType::InvalidChainItem,
                    position
                })
            }
        }

        Ok(())
    }

    //==============================================================================================
    // EXPRESSIONS

    // compile expression
    fn generate_expression(&mut self, position: TokenPosition, token: Box<Syntax>) -> Result<(), CompilerError> {
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
                self.generate_new_object(position, name, input)?
            }

            Syntax::DotChain { position, start, chain } => {
                self.generate_chain(position, start, chain)?;
            }

            Syntax::Call { position, name, input } => {
                self.generate_call(position, name, input)?;
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

            // handle unreadable token and print what it is
            _ => return Err(CompilerError {
                error: CompilerErrorType::InvalidExpressionItem(token),
                position
            })
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_declare_var() {
        let input = vec![
            Syntax::Variable {
                position: TokenPosition { line: 1, column: 1 },
                name: String::from("a"),
                as_type: None,
                value: None,
            }
        ];

        let f = super::Function::new(TokenPosition::default(), String::from("test"), vec![], input);

        match f {
            Ok(f) => {
                assert_eq!(f.instructions, vec![
                    super::Instruction::SetVariableBuffer(1),
                    super::Instruction::PushStackTrace(super::StackTrace {
                        line: 0,
                        file: String::from("unknown"),
                        function: String::from("test"),
                    }),
                    super::Instruction::PushNull,
                    super::Instruction::MoveToLocalVariable(0),
                    super::Instruction::Return,
                    super::Instruction::PopStackTrace,
                ]);
                assert_eq!(f.variables.len(), 1);
                assert_eq!(f.variables.get("a").unwrap().slot_index, 0);
            }
            Err(_) => assert!(f.is_ok())
        }
    }

    #[test]
    fn test_error_when_assign_to_undeclared_var() {
        let input = vec![
            Syntax::Assign {
                position: TokenPosition { line: 1, column: 1 },
                ident: Box::new(Syntax::Identifier {
                    position: TokenPosition { line: 1, column: 1 },
                    name: String::from("a"),
                }),
                value: Box::new(Syntax::Integer(1)),
            }
        ];

        let f = super::Function::new(TokenPosition::default(), String::from("test"), vec![], input);

        assert!(f.is_err());
        assert_eq!(f.err().unwrap(), CompilerError {
            error: CompilerErrorType::VariableNotDeclared(String::from("a")),
            position: TokenPosition { line: 1, column: 1 },
        });
    }
}