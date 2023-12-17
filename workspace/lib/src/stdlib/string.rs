use crate::common::error::{NativeFunctionError, ScriptError};
use crate::common::variant::Variant;
use crate::compiler::script::CONSTRUCTOR_NAME;
use crate::{generic_native_class, script_native_function_error};
use crate::stdlib::{INTERNAL_CLASS_VALUE, PARAM_0, PARAM_1};
use std::collections::HashMap;
use crate::vm::thread::Thread;

pub fn compile_string_class(t: &mut Thread) {

    // add native functions
    t.add_native_function("std_string_constructor", string_constructor);
    t.add_native_function("std_string_length", string_constructor);

    // add class
    let mut class = generic_native_class!();
    class.insert(String::from(CONSTRUCTOR_NAME), Variant::NativeFunctionRef(String::from("std_string_constructor")));
    class.insert(String::from("length"), Variant::NativeFunctionRef(String::from("std_string_length")));
    t.add_global("String", Variant::Class(class));

}

fn string_constructor(p: Vec<Variant>) -> Result<Option<Variant>, ScriptError> {

    // this
    let Some(Variant::Object(this)) = p.get(PARAM_0) else {
        return script_native_function_error!(NativeFunctionError::InvalidSelf)
    };

    // if no parameters then return
    if p.len() > 1 {
        if let Variant::String(initial_value) = p[PARAM_1].clone() {
            this.borrow_mut().insert(INTERNAL_CLASS_VALUE.to_string(), Variant::String(initial_value));
        }
    }

    Ok(Some(Variant::Object(this.clone())))

}

fn string_length(p: Vec<Variant>) -> Result<Option<Variant>, ScriptError> {

    // this
    let Some(Variant::Object(this)) = p.get(PARAM_0) else {
        return script_native_function_error!(NativeFunctionError::InvalidSelf)
    };

    if let Variant::String(v) = this.borrow().get(INTERNAL_CLASS_VALUE).unwrap() {
        return Ok(Some(Variant::Integer(v.len() as i64)));
    }

    script_native_function_error!(NativeFunctionError::InvalidNativeFunction(format!("{}: {}", file!(), line!())))

}

#[cfg(test)]
mod tests {
    use crate::common::variant::Variant;
    use crate::stdlib::INTERNAL_CLASS_VALUE;
    use crate::stdlib::string::{string_constructor, string_length};
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::rc::Rc;

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

            },
            _ => assert!(false, "constructor failed")
        }

    }

    #[test]
    fn test_length() {
        let input = vec![construct_object!()];
        assert_eq!(string_length(input), Ok(Some(Variant::Integer(TEST_STRING.len() as i32))));
    }

}