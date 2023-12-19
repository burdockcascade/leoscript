use crate::common::error::ScriptError;
use crate::stdlib::dictionary::compile_dictionary_class;
use crate::stdlib::math::compile_math_module;
use crate::stdlib::string::compile_string_class;
use crate::vm::thread::Thread;

mod math;
mod dictionary;
mod string;

const INTERNAL_CLASS_VALUE: &str = "#value";
const PARAM_0: usize = 0;
const PARAM_1: usize = 1;
const PARAM_2: usize = 2;


// generic class as hashmap with internal class value
#[macro_export]
macro_rules! generic_native_class {
    () => {
        {
            let mut class = HashMap::new();
            class.insert(String::from(INTERNAL_CLASS_VALUE), Variant::Map(HashMap::new()));
            class
        }
    };
}

pub fn add_standard_library(t: &mut Thread) -> Result<(), ScriptError> {
    t.add_native_function("println", |p| {
        println!("{}", p[PARAM_0]);
        Ok(None)
    });

    compile_string_class(t);
    compile_math_module(t);
    compile_dictionary_class(t);

    Ok(())
}

