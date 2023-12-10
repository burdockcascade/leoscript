use std::collections::HashMap;

use crate::common::variant::Variant;
use crate::script_native_function_error;
use crate::stdlib::{PARAM_1, PARAM_2};
use crate::common::error::{NativeFunctionError, ScriptError};

pub fn compile_math_module() -> Variant {

    let mut mhash = HashMap::new();

    // min
    mhash.insert(String::from("min"), Variant::NativeFunction(|p| {
        if let Variant::Integer(i) = p[PARAM_1] {
            if let Variant::Integer(j) = p[PARAM_2] {
                return Ok(Some(Variant::Integer(i.min(j))));
            }
        }
        script_native_function_error!(NativeFunctionError::UnknownParameterToken)
    }));

    // max
    mhash.insert(String::from("max"), Variant::NativeFunction(|p| {
        if let Variant::Integer(i) = p[PARAM_1] {
            if let Variant::Integer(j) = p[PARAM_2] {
                return Ok(Some(Variant::Integer(i.max(j))));
            }
        }
        script_native_function_error!(NativeFunctionError::UnknownParameterToken)
    }));

    // sqrt
    mhash.insert(String::from("sqrt"), Variant::NativeFunction(|p| {
        if let Variant::Integer(i) = p[PARAM_1] {
            return Ok(Some(Variant::Float((i as f32).sqrt())));
        }
        script_native_function_error!(NativeFunctionError::UnknownParameterToken)
    }));

    // abs
    mhash.insert(String::from("abs"), Variant::NativeFunction(|p| {
        if let Variant::Integer(i) = p[PARAM_1] {
            return Ok(Some(Variant::Integer(i.abs())));
        }
        script_native_function_error!(NativeFunctionError::UnknownParameterToken)
    }));

    Variant::Module(mhash)

}