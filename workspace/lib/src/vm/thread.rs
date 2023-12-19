use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use log::debug;

use crate::common::counter::Counter;
use crate::common::error::{RuntimeError, ScriptError};
use crate::common::instruction::Instruction;
use crate::common::NativeFunctionType;
use crate::common::program::Program;
use crate::common::stacktrace::StackTrace;
use crate::common::variant::Variant;
use crate::script_runtime_error;

const FP_OFFSET: usize = 1;

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
                return Err(ScriptError::RuntimeError {
                    trace: Some($trace),
                    error: RuntimeError::ExpectedValueOnStack,
                });
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
                        return Err(ScriptError::RuntimeError {
                            trace: Some($trace),
                            error: RuntimeError::ExpectedValueOnStack,
                        });
                    }
                }
            }
            _ => {
                return Err(ScriptError::RuntimeError {
                    trace: Some($trace),
                    error: RuntimeError::ExpectedValueOnStack,
                });
            }
        }
    };
}

impl Thread {
    pub fn load_program(program: Program) -> Result<Self, ScriptError> {
        if program.instructions.is_empty() {
            return Err(ScriptError::RuntimeError {
                trace: None,
                error: RuntimeError::NoInstructions,
            });
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

    pub fn run(&mut self, entry: &str, parameters: Option<Vec<Variant>>) -> Result<ExecutionResult, ScriptError> {

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

    fn execute(&mut self, entry: &str, parameters: Option<Vec<Variant>>) -> Result<Option<Variant>, ScriptError> {
        let mut stack: Vec<Variant> = Vec::with_capacity(64);
        let mut trace: Vec<StackTrace> = Vec::with_capacity(64);

        let mut ip: usize;
        let mut fp: usize = 1;

        if let Some(Variant::FunctionPointer(fpointer)) = self.program.globals.get(entry) {
            stack.push(Variant::ReturnPointer(0));
            stack.push(Variant::FramePointer(0));
            ip = *fpointer;
        } else {
            return Err(ScriptError::RuntimeError {
                trace: Some(trace),
                error: RuntimeError::EntryPointNotFound(entry.to_string()),
            });
        }

        // push initial parameters onto stack
        if let Some(parameters) = parameters {
            for parameter in parameters {
                stack.push(parameter);
            }
        }

        //debug!("Globals: {:#?}", self.program.globals);

        // loop over instructions
        loop {
            debug!("[{:?}] {:?}", ip, self.program.instructions[ip]);
            debug!("stack: {:?}", stack);
            // debug!("fp: {}", fp);

            // get instruction
            let Some(instruction) = self.program.instructions.get(ip) else {
                return Err(ScriptError::RuntimeError {
                    trace: Some(trace),
                    error: RuntimeError::InstructionPointerOutOfBounds(ip),
                });
            };

            match instruction {
                Instruction::NoOperation => {
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
                        return script_runtime_error!(trace, RuntimeError::ExpectedIntegerOnStack);
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

                Instruction::PushFunctionRef(value) => {
                    stack.push(Variant::FunctionRef(value.clone()));
                    ip += 1;
                }

                Instruction::PushFunctionPointer(value) => {
                    stack.push(Variant::FunctionPointer(value.clone()));
                    ip += 1;
                }

                Instruction::SetVariableBuffer(size) => {

                    // pad stack with nulls
                    for _ in 0..*size {
                        stack.push(Variant::Null);
                    }

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
                    let stack_index = fp + *index + FP_OFFSET;

                    if stack_index < stack.len() {
                        stack[stack_index] = value;
                        ip += 1;
                    } else {
                        return script_runtime_error!(trace, RuntimeError::InvalidStackIndex { index: stack_index, size: stack.len() });
                    }
                }

                // get value from variable and push onto stack
                Instruction::LoadLocalVariable(index) => {
                    let value = stack[fp + *index + FP_OFFSET].clone();
                    stack.push(value);
                    ip += 1;
                }

                Instruction::LoadGlobal(name) => {
                    if let Some(function_ref) = self.program.globals.get(name) {
                        stack.push(function_ref.clone());
                        ip += 1;
                    } else {
                        return script_runtime_error!(trace, RuntimeError::GlobalNotFound(name.clone()));
                    }
                }

                //==================================================================================
                // FUNCTIONS

                Instruction::LoadMember(name) => {
                    match pop_tos!(stack, trace) {
                        Variant::Object(object) => {
                            let borrowed_object = object.borrow();
                            if let Some(function_ref) = borrowed_object.get(name) {
                                stack.push(function_ref.clone());
                                stack.push(Variant::Object(object.clone()));
                                ip += 1;
                            } else {
                                return script_runtime_error!(trace, RuntimeError::MethodNotFound(name.clone()));
                            }
                        }
                        Variant::Class(class_template) => {
                            if let Some(function_ref) = class_template.get(name) {
                                stack.push(function_ref.clone());
                                stack.push(Variant::Class(class_template.clone()));
                                ip += 1;
                            } else {
                                return script_runtime_error!(trace, RuntimeError::MethodNotFound(name.clone()));
                            }
                        }
                        Variant::Module(module) => {
                            if let Some(function_ref) = module.get(name) {
                                stack.push(function_ref.clone());
                                stack.push(Variant::Module(module.clone()));
                                ip += 1;
                            } else {
                                return script_runtime_error!(trace, RuntimeError::MethodNotFound(name.clone()));
                            }
                        }
                        _ => {
                            return script_runtime_error!(trace, RuntimeError::ExpectedObjectOnStack);
                        }
                    }
                }

                Instruction::Call(arg_len) => {

                    // cut args from stack and then reverse order
                    let mut args: Vec<Variant> = Vec::default();
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
                            continue;
                        }

                        // do nothing
                        Variant::FunctionPointer(_) => tos,

                        _ => return script_runtime_error!(trace, RuntimeError::InvalidFunctionOnStack(tos))
                    };

                    match tos {

                        // jump to function pointer
                        Variant::FunctionPointer(fptr) => {

                            // set return pointer
                            stack.push(Variant::ReturnPointer(ip + 1));

                            // set frame pointer
                            stack.push(Variant::FramePointer(fp));

                            // push new frame onto frames
                            fp = stack.len() - 1;

                            // push args onto stack
                            for arg in args {
                                stack.push(arg);
                            }

                            // set instruction pointer to function
                            ip = fptr;
                        }

                        // invalid call destination
                        _ => return script_runtime_error!(trace, RuntimeError::InvalidCallDestination(tos))
                    }
                }

                Instruction::Return => {

                    // reduce stack to fp
                    while stack.len() > (fp + 1) {
                        stack.pop();
                    }

                    // remove last frame
                    match pop_tos!(stack, trace) {
                        Variant::FramePointer(frm) => fp = frm,
                        _ => return script_runtime_error!(trace, RuntimeError::InvalidFramePointer),
                    }

                    if fp == 0 {
                        return Ok(None);
                    }

                    // get return position or return from self.program
                    match pop_tos!(stack, trace) {
                        Variant::ReturnPointer(ptr) => ip = ptr,
                        _ => return script_runtime_error!(trace, RuntimeError::InvalidReturnPointer),
                    }
                }

                Instruction::ReturnWithValue => {

                    // pop return value from stack
                    let return_value = pop_tos!(stack, trace);

                    // reduce stack to fp
                    while stack.len() > (fp + 1) {
                        stack.pop();
                    }

                    // remove last frame
                    match pop_tos!(stack, trace) {
                        Variant::FramePointer(frm) => fp = frm,
                        _ => return script_runtime_error!(trace, RuntimeError::InvalidFramePointer),
                    }

                    if fp == 0 {
                        return Ok(Some(return_value));
                    }

                    // get return position or return from self.program
                    match pop_tos!(stack, trace) {
                        Variant::ReturnPointer(ptr) => ip = ptr,
                        _ => return script_runtime_error!(trace, RuntimeError::InvalidReturnPointer),
                    }


                    // push return value onto stack
                    stack.push(return_value);
                }


                //==================================================================================
                // Objects

                Instruction::CreateObject => {
                    let tos = pop_tos!(stack, trace);

                    // get class
                    let Variant::Class(class_template) = tos else {
                        return script_runtime_error!(trace, RuntimeError::ExpectedClassOnStack);
                    };

                    // create new object
                    let new_object = Variant::Object(Rc::new(RefCell::new(class_template.clone())));

                    // push new object onto stack
                    stack.push(new_object.clone());

                    ip += 1;
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
                            _ => return script_runtime_error!(trace, RuntimeError::InvalidDictionaryKey)
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
                            if let Variant::Integer(index) = key {
                                match items.get(index as usize) {
                                    Some(v) => stack.push(v.clone()),
                                    None => return Err(ScriptError::RuntimeError {
                                        trace: Some(trace),
                                        error: RuntimeError::InvalidArrayIndex,
                                    })
                                }
                            } else {
                                panic!("can not get index on non-integer {}", key)
                            }
                        }
                        Variant::Object(obj) => {
                            if let Variant::String(index) = key {
                                let items_borrowed = obj.borrow();
                                match items_borrowed.get(index.as_str()) {
                                    Some(v) => stack.push(v.clone()),
                                    None => return script_runtime_error!(trace, RuntimeError::InvalidObjectMember),
                                }
                            } else {
                                panic!("can not get index on non-string {}", key)
                            }
                        }
                        Variant::Module(obj) => {
                            if let Variant::String(index) = key {
                                match obj.get(index.as_str()) {
                                    Some(v) => stack.push(v.clone()),
                                    None => return script_runtime_error!(trace, RuntimeError::InvalidObjectMember),
                                }
                            } else {
                                panic!("can not get index on non-string {}", key)
                            }
                        }
                        Variant::Enum(obj) => {
                            if let Variant::String(index) = key {
                                match obj.get(index.as_str()) {
                                    Some(v) => stack.push(Variant::Integer(*v as i64)),
                                    None => return script_runtime_error!(trace, RuntimeError::InvalidObjectMember),
                                }
                            }
                        }
                        _ => panic!("can not get index on non-collection {}", collection)
                    }

                    ip += 1;
                }

                Instruction::SetCollectionItem => {
                    let key = pop_tos!(stack, trace);
                    let value = pop_tos!(stack, trace);
                    let collection = pop_tos!(stack, trace);

                    match collection {
                        Variant::Array(mut items) => {
                            if let Variant::Integer(index) = key {
                                items[index as usize] = value;
                            } else {
                                return script_runtime_error!(trace, RuntimeError::InvalidArrayIndex);
                            }
                        }
                        Variant::Object(obj) => {
                            if let Variant::String(index) = key {
                                let mut items_borrowed = obj.borrow_mut();
                                items_borrowed.insert(index, value);
                            } else {
                                return script_runtime_error!(trace, RuntimeError::InvalidObjectMember);
                            }
                        }
                        _ => return script_runtime_error!(trace, RuntimeError::InvalidCollection),
                    }

                    ip += 1;
                }


                //==================================================================================
                // ITERATION

                Instruction::IteratorInit => {
                    let Variant::Integer(start) = pop_tos!(stack, trace) else {
                        return script_runtime_error!(trace, RuntimeError::ExpectedIntegerOnStack);
                    };

                    let Variant::Integer(step) = pop_tos!(stack, trace) else {
                        return script_runtime_error!(trace, RuntimeError::ExpectedIntegerOnStack);
                    };

                    let target = pop_tos!(stack, trace);

                    let cntr = Counter::new(start, step, target);
                    stack.push(Variant::Iterator(Box::from(cntr)));

                    ip += 1;
                }

                Instruction::IteratorNext(var_slot) => {

                    // get counter from stack
                    let Variant::Iterator(mut counter) = pop_tos!(stack, trace) else {
                        return script_runtime_error!(trace, RuntimeError::ExpectedIteratorOnStack);
                    };

                    match counter.next() {
                        Some(value) => {
                            stack[fp + *var_slot + FP_OFFSET] = value;

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

                _ => return script_runtime_error!(trace, RuntimeError::InstructionNotImplemented(instruction.clone())),
            }
        }
    }
}