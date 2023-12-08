use crate::common::error::ScriptError;
use crate::stdlib::dictionary::compile_dictionary_class;
use crate::stdlib::math::compile_math_module;
use crate::vm::thread::Thread;

mod math;
mod dictionary;

const PARAM_SELF: usize = 0;
const PARAM_1: usize = 1;
const PARAM_2: usize = 2;
const PARAM_3: usize = 3;
const PARAM_4: usize = 4;

pub fn add_standard_library(t: &mut Thread) -> Result<(), ScriptError> {

    t.add_global("Math", compile_math_module())?;
    t.add_global("Dictionary", compile_dictionary_class())?;

    Ok(())
}