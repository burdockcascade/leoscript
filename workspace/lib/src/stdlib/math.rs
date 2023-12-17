use std::collections::HashMap;

use crate::common::variant::Variant;
use crate::script_native_function_error;
use crate::stdlib::{PARAM_1, PARAM_2};
use crate::common::error::{NativeFunctionError, ScriptError};
use crate::vm::thread::Thread;

pub fn compile_math_module(t: &mut Thread) {

    t.add_native_function("std_math_max", math_max);
    t.add_native_function("std_math_min", math_min);
    t.add_native_function("std_math_sqrt", math_sqrt);
    t.add_native_function("std_math_abs", math_abs);

    let mut mhash = HashMap::new();
    mhash.insert(String::from("min"), Variant::NativeFunctionRef(String::from("std_math_min")));
    mhash.insert(String::from("max"), Variant::NativeFunctionRef(String::from("std_math_max")));
    mhash.insert(String::from("sqrt"), Variant::NativeFunctionRef(String::from("std_math_sqrt")));
    mhash.insert(String::from("abs"), Variant::NativeFunctionRef(String::from("std_math_abs")));
    t.add_global("Math", Variant::Module(mhash));
}

fn math_max(p: Vec<Variant>) -> Result<Option<Variant>, ScriptError> {
    if let Variant::Integer(i) = p[PARAM_1] {
        if let Variant::Integer(j) = p[PARAM_2] {
            return Ok(Some(Variant::Integer(i.max(j))));
        }
    }
    script_native_function_error!(NativeFunctionError::UnknownParameterToken)
}

fn math_min(p: Vec<Variant>) -> Result<Option<Variant>, ScriptError> {
    if let Variant::Integer(i) = p[PARAM_1] {
        if let Variant::Integer(j) = p[PARAM_2] {
            return Ok(Some(Variant::Integer(i.min(j))));
        }
    }
    script_native_function_error!(NativeFunctionError::UnknownParameterToken)
}

fn math_sqrt(p: Vec<Variant>) -> Result<Option<Variant>, ScriptError> {
    if let Variant::Integer(i) = p[PARAM_1] {
        return Ok(Some(Variant::Float((i as f64).sqrt())));
    }
    script_native_function_error!(NativeFunctionError::UnknownParameterToken)
}

fn math_abs(p: Vec<Variant>) -> Result<Option<Variant>, ScriptError> {
    if let Variant::Integer(i) = p[PARAM_1] {
        return Ok(Some(Variant::Integer(i.abs())));
    }
    script_native_function_error!(NativeFunctionError::UnknownParameterToken)
}

#[cfg(test)]
mod tests {
    use crate::common::variant::Variant;
    use crate::stdlib::math::{math_abs, math_max, math_min, math_sqrt};

    #[test]
    fn test_math_max() {

        let input = vec![
            Variant::Module(std::collections::HashMap::new()),
            Variant::Integer(1),
            Variant::Integer(2),
        ];

        let expected = Variant::Integer(2);

        match math_max(input) {
            Ok(Some(v)) => assert_eq!(v, expected),
            _ => assert!(false),
        }

    }

    #[test]
    fn test_math_min() {

        let input = vec![
            Variant::Module(std::collections::HashMap::new()),
            Variant::Integer(1),
            Variant::Integer(2),
        ];

        let expected = Variant::Integer(1);

        match math_min(input) {
            Ok(Some(v)) => assert_eq!(v, expected),
            _ => assert!(false),
        }

    }

    #[test]
    fn test_math_abs() {

        let input = vec![
            Variant::Module(std::collections::HashMap::new()),
            Variant::Integer(-1),
        ];

        let expected = Variant::Integer(1);

        match math_abs(input) {
            Ok(Some(v)) => assert_eq!(v, expected),
            _ => assert!(false),
        }

    }

    #[test]
    fn test_math_sqrt() {

        let input = vec![
            Variant::Module(std::collections::HashMap::new()),
            Variant::Integer(4),
        ];

        let expected = Variant::Float(2.0);

        match math_sqrt(input) {
            Ok(Some(v)) => assert_eq!(v, expected),
            _ => assert!(false),
        }

    }



}