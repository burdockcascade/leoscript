use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::runtime::error::RuntimeError;
use crate::runtime::ir::counter::Counter;
use crate::runtime::ir::instruction::Instruction;
use crate::runtime::ir::program::Program;
use crate::runtime::ir::stacktrace::StackTrace;
use crate::runtime::ir::variant::{CLASS_CONSTRUCTOR_NAME, Variant};
use crate::runtime::stdlib::NativeFunctionType;
use crate::runtime::vm::frame::Frame;

#[derive(Debug, PartialEq)]
pub struct Thread {
    program: Program,
    native_functions: HashMap<String, NativeFunctionType>,
    options: ThreadOptions,
}

#[derive(Debug, PartialEq)]
struct ThreadOptions {
    stack_trace_enabled: bool,
}

impl Default for ThreadOptions {
    fn default() -> Self {
        ThreadOptions {
            stack_trace_enabled: false,
        }
    }
}

pub struct ExecutionResult {
    pub output: Option<Variant>,
    pub execution_time: std::time::Duration,
}

// macro that pops the top of the stack or returns an error with the trace
macro_rules! pop_tos {
    ($stack:ident, $trace:ident) => {
        match $stack.pop() {
            Some(value) => value,
            _ => {
                return Err(RuntimeError::ExpectedValueOnStack);
            }
        }
    };
}

// macro that pops the top 2 items from the stack
macro_rules! pop_tos2 {
    ($stack:ident, $trace:ident) => {
        match $stack.pop() {
            Some(rhs) => {
                match $stack.pop() {
                    Some(lhs) => (rhs, lhs),
                    _ => {
                        return Err(RuntimeError::ExpectedValueOnStack);
                    }
                }
            }
            _ => {
                return Err(RuntimeError::ExpectedValueOnStack);
            }
        }
    };
}

impl Thread {
    pub fn load_program(program: Program) -> Result<Self, RuntimeError> {
        if program.instructions.is_empty() {
            return Err(RuntimeError::NoInstructions);
        }

        Ok(Thread {
            program,
            native_functions: Default::default(),
            options: Default::default(),
        })
    }

    // add native function to global scope
    pub fn add_native_function(&mut self, name: &str, callback: NativeFunctionType) {
        self.native_functions.insert(name.to_string(), callback);
    }

    pub fn add_global(&mut self, name: &str, value: Variant) {
        self.program.globals.insert(name.to_string(), value);
    }

    pub fn run(&mut self, entry: &str, parameters: Option<Vec<Variant>>) -> Result<ExecutionResult, RuntimeError> {

        // start timer
        let start_execution = std::time::Instant::now();

        // run
        let result = self.execute(entry, parameters);

        // end timer
        let end_execution = std::time::Instant::now();

        // return result
        match result {
            Ok(v) => Ok(ExecutionResult {
                output: v,
                execution_time: end_execution - start_execution,
            }),
            Err(e) => Err(e),
        }
    }

    fn execute(&mut self, entry: &str, parameters: Option<Vec<Variant>>) -> Result<Option<Variant>, RuntimeError> {
        let mut frames: Vec<Frame> = Vec::with_capacity(64);
        let mut stack: Vec<Variant> = Vec::with_capacity(64);
        let mut trace: Vec<StackTrace> = Vec::with_capacity(64);
        let mut current_frame = Frame::default();

        let mut ip: usize;

        if let Some(Variant::FunctionPointer(fpointer)) = self.program.globals.get(entry) {
            ip = *fpointer;
        } else {
            return Err(RuntimeError::EntryPointNotFound(entry.to_string()));
        }

        // push initial parameters onto stack
        if let Some(parameters) = parameters {
            for parameter in parameters {
                current_frame.variables.push(parameter)
            }
        }

        //debug!("Globals: {:#?}", self.program.globals);

        // loop over instructions
        loop {

            // println!("---");
            // println!("frames: {:?}", frames);
            // println!("rp: {:?}", current_frame.return_address);
            // println!("variables: {:?}", current_frame.variables);
            // println!("stack: {:?}", stack);
            // println!("[{:?}] {:?}", ip, self.program.instructions[ip]);

            // get instruction
            let Some(instruction) = self.program.instructions.get(ip) else {
                return Err(RuntimeError::InstructionPointerOutOfBounds(ip));
            };

            match instruction {
                Instruction::NoOperation => {
                    ip += 1;
                }

                Instruction::Debug(value) => {
                    println!("DEBUG: {:?}", value);
                    ip += 1;
                }

                Instruction::Print => {
                    let value = pop_tos!(stack, trace);
                    print!("{}", value);
                    ip += 1;
                }

                Instruction::Sleep => {
                    if let Variant::Integer(value) = pop_tos!(stack, trace) {
                        std::thread::sleep(std::time::Duration::from_millis(value as u64));
                    } else {
                        return Err(RuntimeError::ExpectedIntegerOnStack);
                    }
                    ip += 1;
                }

                //==================================================================================
                // STACK

                Instruction::PushNull => {
                    stack.push(Variant::Null);
                    ip += 1;
                }

                Instruction::PushInteger(value) => {
                    stack.push(Variant::Integer(*value));
                    ip += 1;
                }

                Instruction::PushFloat(value) => {
                    stack.push(Variant::Float(*value));
                    ip += 1;
                }

                Instruction::PushBool(value) => {
                    stack.push(Variant::Bool(*value));
                    ip += 1;
                }

                Instruction::PushString(value) => {
                    stack.push(Variant::String(value.clone()));
                    ip += 1;
                }

                Instruction::PushIdentifier(value) => {
                    stack.push(Variant::Identifier(value.clone()));
                    ip += 1;
                }

                Instruction::PushFunctionRef(value) => {
                    stack.push(Variant::FunctionRef(value.clone()));
                    ip += 1;
                }

                Instruction::PushFunctionPointer(value) => {
                    stack.push(Variant::FunctionPointer(value.clone()));
                    ip += 1;
                }


                //==================================================================================
                // STACK TRACE

                Instruction::PushStackTrace(t) => {
                    if self.options.stack_trace_enabled {
                        trace.push(t.clone());
                    }
                    ip += 1;
                }

                Instruction::PopStackTrace => {
                    if self.options.stack_trace_enabled {
                        trace.pop();
                    }
                    ip += 1;
                }

                //==================================================================================
                // CONTROL FLOW

                Instruction::JumpForward(delta) => {
                    ip += *delta;
                }

                Instruction::JumpBackward(delta) => {
                    ip -= *delta;
                }

                Instruction::JumpForwardIfFalse(delta) => {
                    if let Variant::Bool(false) = pop_tos!(stack, trace) {
                        ip += *delta;
                    } else {
                        ip += 1;
                    }
                }


                //==================================================================================
                // VARIABLES

                // get value from stack and store in variable
                Instruction::MoveToLocalVariable(index) => {
                    let value = pop_tos!(stack, trace);
                    current_frame.set_variable(*index, value)?;
                    ip += 1;
                }

                // get value from variable and push onto stack
                Instruction::LoadLocalVariable(index) => {
                    let value = current_frame.get_variable(*index)?.clone();
                    stack.push(value);
                    ip += 1;
                }

                Instruction::LoadGlobal(name) => {
                    if let Some(function_ref) = self.program.globals.get(name) {
                        stack.push(function_ref.clone());
                        ip += 1;
                    } else {
                        return Err(RuntimeError::GlobalNotFound(name.clone()));
                    }
                }

                //==================================================================================
                // FUNCTIONS

                Instruction::LoadMember(name) => {
                    let parent = pop_tos!(stack, trace);

                    match parent {
                        Variant::Object(object) => {
                            let borrowed_object = object.borrow();
                            match borrowed_object.get(name.as_str()) {
                                Some(v) => {
                                    stack.push(v.clone());
                                    match v {
                                        Variant::FunctionPointer(_) => stack.push(Variant::Object(object.clone())),
                                        Variant::NativeFunctionRef(_) => stack.push(Variant::Object(object.clone())),
                                        _ => {}
                                    }
                                    ip += 1;
                                }
                                None => return Err(RuntimeError::MethodNotFound(name.clone())),
                            }
                        }
                        Variant::Class(class_template) => {
                            match class_template.get(name.as_str()) {
                                Some(function_ref) => {
                                    stack.push(function_ref.clone());
                                    stack.push(Variant::Class(class_template.clone()));
                                    ip += 1;
                                }
                                None => {
                                    return Err(RuntimeError::MethodNotFound(name.clone()));
                                }
                            }
                        }
                        Variant::Module(module) => {
                            match module.get(name.as_str()) {
                                Some(function_ref) => {
                                    stack.push(function_ref.clone());
                                    stack.push(Variant::Module(module.clone()));
                                    ip += 1;
                                }
                                None => {
                                    return Err(RuntimeError::MethodNotFound(name.clone()));
                                }
                            }
                        }
                        _ => {
                            return Err(RuntimeError::ExpectedObjectOnStack);
                        }
                    }
                }

                Instruction::Call(arg_len) => {

                    // cut args from stack and then reverse order
                    let mut args: Vec<Variant> = Vec::with_capacity(*arg_len);
                    for _ in 0..*arg_len {
                        args.push(pop_tos!(stack, trace));
                    }
                    args.reverse();

                    // get function reference
                    let mut tos = pop_tos!(stack, trace);

                    // check if tos is FunctionRef
                    tos = match tos {

                        // is FunctionRef and in globals
                        Variant::FunctionRef(ident) if self.program.globals.contains_key(&ident) => {
                            self.program.globals.get(&ident).unwrap().clone()
                        }

                        Variant::Class(class_template) => {

                            // create new object
                            let new_object = Variant::Object(Rc::new(RefCell::new(class_template.clone())));

                            // push self into arguments
                            args.insert(0, new_object);

                            let Some(fp) = class_template.get(CLASS_CONSTRUCTOR_NAME) else {
                                return Err(RuntimeError::ConstructorNotFound);
                            };

                            fp.clone()
                        }

                        // do nothing
                        Variant::FunctionPointer(_) => tos,
                        Variant::NativeFunctionRef(_) => tos,

                        _ => return Err(RuntimeError::InvalidCallOnStack(tos))
                    };

                    match tos {

                        // is FunctionRef and in native_functions
                        Variant::NativeFunctionRef(ident) if self.native_functions.contains_key(&ident) => {
                            let func = self.native_functions.get(&ident).unwrap();

                            match func(args) {
                                Ok(Some(result)) => {
                                    stack.push(result);
                                }
                                Ok(None) => {}
                                Err(error) => {
                                    return Err(error);
                                }
                            }

                            ip += 1;
                        }

                        // jump to function pointer
                        Variant::FunctionPointer(fptr) => {

                            // stack current frame
                            frames.push(current_frame);

                            // new frame
                            current_frame = Frame {
                                return_address: ip + 1,
                                stack_pointer: stack.len(),
                                variables: Vec::with_capacity(16),
                            };

                            // push args onto stack
                            for arg in args {
                                current_frame.variables.push(arg);
                            }

                            // set instruction pointer to function
                            ip = fptr;
                        }

                        // invalid call destination
                        _ => return Err(RuntimeError::InvalidCallDestination(tos))
                    }
                }

                Instruction::Return { with_value } => {
                    ip = current_frame.return_address;
                    match frames.pop() {
                        Some(frame) => {

                            // cleanup stack after return
                            stack.truncate(current_frame.stack_pointer + (if *with_value { 1 } else { 0 }));

                            // set new frame
                            current_frame = frame;
                        }
                        None => {
                            if *with_value {
                                return Ok(Some(pop_tos!(stack, trace)));
                            } else {
                                return Ok(None);
                            }
                        }
                    }
                }

                //==================================================================================
                // COLLECTIONS

                Instruction::CreateCollectionAsDictionary(size) => {
                    let mut items = HashMap::new();

                    for _ in 0..*size {
                        let value = pop_tos!(stack, trace);
                        let key = pop_tos!(stack, trace);

                        match key {
                            Variant::String(key) => {
                                items.insert(key, value);
                            }
                            _ => return Err(RuntimeError::InvalidDictionaryKey)
                        }
                    }

                    stack.push(Variant::Map(items));

                    ip += 1;
                }

                Instruction::CreateCollectionAsArray(size) => {
                    let mut items = Vec::new();

                    for _ in 0..*size {
                        items.push(pop_tos!(stack, trace));
                    }

                    items.reverse();

                    stack.push(Variant::Array(items));

                    ip += 1;
                }

                Instruction::GetCollectionItem => {
                    let key = pop_tos!(stack, trace);
                    let collection = pop_tos!(stack, trace);

                    match collection {
                        Variant::Array(items) => {
                            match key {
                                Variant::Integer(index) => {
                                    match items.get(index as usize) {
                                        Some(v) => stack.push(v.clone()),
                                        None => return Err(RuntimeError::InvalidArrayIndex)
                                    }
                                }
                                _ => return Err(RuntimeError::InvalidCollectionKey(key))
                            }
                        }
                        Variant::Object(obj) => {
                            match key {
                                Variant::Identifier(index) => {
                                    let items_borrowed = obj.borrow();
                                    match items_borrowed.get(index.as_str()) {
                                        Some(v) => stack.push(v.clone()),
                                        None => return Err(RuntimeError::InvalidObjectMember),
                                    }
                                }
                                _ => return Err(RuntimeError::InvalidCollectionKey(key))
                            }
                        }
                        Variant::Module(obj) => {
                            match key {
                                Variant::Identifier(index) => {
                                    match obj.get(index.as_str()) {
                                        Some(v) => stack.push(v.clone()),
                                        None => return Err(RuntimeError::ModuleIndexNotFound(index)),
                                    }
                                }
                                _ => return Err(RuntimeError::InvalidCollectionKey(key))
                            }
                        }
                        Variant::Enum(obj) => {
                            match key {
                                Variant::Identifier(index) => {
                                    match obj.get(index.as_str()) {
                                        Some(v) => stack.push(Variant::Integer(*v as i64)),
                                        None => return Err(RuntimeError::EnumIndexNotFound(index)),
                                    }
                                }
                                _ => return Err(RuntimeError::InvalidCollectionKey(key))
                            }
                        }
                        _ => return Err(RuntimeError::InvalidCollection)
                    }

                    ip += 1;
                }

                Instruction::SetCollectionItem => {
                    let value = pop_tos!(stack, trace);
                    let key = pop_tos!(stack, trace);
                    let collection = pop_tos!(stack, trace);

                    match collection {
                        Variant::Array(mut items) => {
                            if let Variant::Integer(index) = key {
                                items[index as usize] = value;
                            } else {
                                return Err(RuntimeError::InvalidArrayIndex);
                            }
                        }
                        Variant::Object(obj) => {
                            if let Variant::Identifier(index) = key {
                                let mut items_borrowed = obj.borrow_mut();
                                items_borrowed.insert(index, value);
                            } else {
                                return Err(RuntimeError::InvalidObjectMember);
                            }
                        }
                        Variant::Identifier(index) => unimplemented!("set collection item for module"),
                        _ => return Err(RuntimeError::InvalidCollection),
                    }

                    ip += 1;
                }

                Instruction::SetArrayItem => {
                    let value = pop_tos!(stack, trace);
                    let key = pop_tos!(stack, trace);
                    let collection = pop_tos!(stack, trace);

                    match collection {
                        Variant::Array(mut items) => {
                            if let Variant::Integer(index) = key {
                                items[index as usize] = value;
                                stack.push(Variant::Array(items));
                            } else {
                                return Err(RuntimeError::InvalidArrayIndex);
                            }
                        }
                        _ => return Err(RuntimeError::InvalidCollection),
                    }

                    ip += 1;
                }


                //==================================================================================
                // ITERATION

                Instruction::IteratorInit => {
                    let Variant::Integer(start) = pop_tos!(stack, trace) else {
                        return Err(RuntimeError::ExpectedIntegerOnStack);
                    };

                    let Variant::Integer(step) = pop_tos!(stack, trace) else {
                        return Err(RuntimeError::ExpectedIntegerOnStack);
                    };

                    let target = pop_tos!(stack, trace);

                    let cntr = Counter::new(start, step, target);
                    stack.push(Variant::Iterator(Box::from(cntr)));

                    ip += 1;
                }

                Instruction::IteratorNext(var_slot) => {

                    // get counter from stack
                    let Variant::Iterator(mut counter) = pop_tos!(stack, trace) else {
                        return Err(RuntimeError::ExpectedIteratorOnStack);
                    };

                    match counter.next() {
                        Some(value) => {
                            current_frame.set_variable(*var_slot, value)?;

                            stack.push(Variant::Iterator(counter));
                            stack.push(Variant::Bool(true))
                        }
                        None => {
                            stack.push(Variant::Bool(false))
                        }
                    }

                    ip += 1
                }


                //==================================================================================
                // ARITHMETIC

                Instruction::Add => {
                    let (v2, v1) = pop_tos2!(stack, trace);
                    stack.push(v1 + v2);
                    ip += 1;
                }

                Instruction::Sub => {
                    let (v2, v1) = pop_tos2!(stack, trace);
                    stack.push(v1 - v2);
                    ip += 1;
                }

                Instruction::Multiply => {
                    let (v2, v1) = pop_tos2!(stack, trace);
                    stack.push(v1 * v2);
                    ip += 1;
                }

                Instruction::Divide => {
                    let (v2, v1) = pop_tos2!(stack, trace);
                    stack.push(v1 / v2);
                    ip += 1;
                }

                Instruction::Pow => {
                    let (v2, v1) = pop_tos2!(stack, trace);
                    stack.push(v1.pow(v2));
                    ip += 1;
                }

                Instruction::Not => {
                    let v = pop_tos!(stack, trace);
                    stack.push(!v);
                    ip += 1;
                }

                Instruction::And => {
                    let (v2, v1) = pop_tos2!(stack, trace);
                    stack.push(v1 & v2);
                    ip += 1;
                }

                Instruction::Or => {
                    let (v2, v1) = pop_tos2!(stack, trace);
                    stack.push(v1 | v2);
                    ip += 1;
                }

                //==================================================================================
                // OPERANDS

                Instruction::Equal => {
                    let (v2, v1) = pop_tos2!(stack, trace);
                    stack.push(Variant::Bool(v1 == v2));
                    ip += 1;
                }

                Instruction::NotEqual => {
                    let (v2, v1) = pop_tos2!(stack, trace);
                    stack.push(Variant::Bool(v1 != v2));
                    ip += 1;
                }

                Instruction::LessThan => {
                    let (v2, v1) = pop_tos2!(stack, trace);
                    stack.push(Variant::Bool(v1 < v2));
                    ip += 1;
                }

                Instruction::LessThanOrEqual => {
                    let (v2, v1) = pop_tos2!(stack, trace);
                    stack.push(Variant::Bool(v1 <= v2));
                    ip += 1;
                }

                Instruction::GreaterThan => {
                    let (v2, v1) = pop_tos2!(stack, trace);
                    stack.push(Variant::Bool(v1 > v2));
                    ip += 1;
                }

                Instruction::GreaterThanOrEqual => {
                    let (v2, v1) = pop_tos2!(stack, trace);
                    stack.push(Variant::Bool(v1 >= v2));
                    ip += 1;
                }

                _ => return Err(RuntimeError::InstructionNotImplemented(instruction.clone())),
            }
        }
    }
}