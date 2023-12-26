use std::collections::HashMap;

use crate::generic_native_class;
use crate::runtime::error::RuntimeError;
use crate::runtime::ir::variant::Variant;
use crate::runtime::stdlib::{CONSTRUCTOR_NAME, INTERNAL_CLASS_VALUE, PARAM_0, PARAM_1};
use crate::runtime::vm::thread::Thread;

pub fn compile_string_class(t: &mut Thread) {

    // add native functions
    t.add_native_function("std_string_constructor", string_constructor);
    t.add_native_function("std_string_length", string_length);

    // add class
    let mut class = generic_native_class!();
    class.insert(String::from(CONSTRUCTOR_NAME), Variant::NativeFunctionRef(String::from("std_string_constructor")));
    class.insert(String::from("length"), Variant::NativeFunctionRef(String::from("std_string_length")));
    t.add_global("String", Variant::Class(class));
}

fn string_constructor(p: Vec<Variant>) -> Result<Option<Variant>, RuntimeError> {

    // this
    let Some(Variant::Object(this)) = p.get(PARAM_0) else {
        return Err(RuntimeError::ExpectedSelfForNativeFunction);
    };

    // if no parameters then return
    if p.len() > 1 {
        if let Variant::String(initial_value) = p[PARAM_1].clone() {
            this.borrow_mut().insert(INTERNAL_CLASS_VALUE.to_string(), Variant::String(initial_value));
        }
    }

    Ok(Some(Variant::Object(this.clone())))
}

fn string_length(p: Vec<Variant>) -> Result<Option<Variant>, RuntimeError> {

    // this
    let Some(Variant::Object(this)) = p.get(PARAM_0) else {
        return Err(RuntimeError::ExpectedSelfForNativeFunction);
    };

    if let Variant::String(v) = this.borrow().get(INTERNAL_CLASS_VALUE).unwrap() {
        return Ok(Some(Variant::Integer(v.len() as i64)));
    }

    Err(RuntimeError::InvalidNativeFunction(format!("{}: {}", file!(), line!())))
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use super::*;

    const TEST_STRING: &str = "I am a test string";

    macro_rules! construct_object {
        () => {
            {
                let mut class = HashMap::new();
                class.insert(String::from(INTERNAL_CLASS_VALUE), Variant::String(String::from(TEST_STRING)));

                Variant::Object(Rc::new(RefCell::new(class)))
            }
        };
    }

    #[test]
    fn test_constructor() {
        let input = vec![
            construct_object!(),
        ];

        match string_constructor(input) {
            Ok(Some(Variant::Object(obj))) => {
                if let Some(value) = obj.borrow().get(INTERNAL_CLASS_VALUE) {
                    if let Variant::String(v) = value {
                        assert_eq!(v, TEST_STRING);
                    } else {
                        assert!(false, "value not a string");
                    }
                } else {
                    assert!(false, "internal value not found");
                }
            }
            _ => assert!(false, "constructor failed")
        }
    }

    #[test]
    fn test_length() {
        let input = vec![construct_object!()];
        assert_eq!(string_length(input), Ok(Some(Variant::Integer(TEST_STRING.len() as i64))));
    }
}