use std::collections::HashMap;

use crate::generic_native_class;
use crate::runtime::error::RuntimeError;
use crate::runtime::ir::variant::Variant;
use crate::runtime::stdlib::{CONSTRUCTOR_NAME, INTERNAL_CLASS_VALUE, PARAM_0, PARAM_1, PARAM_2};
use crate::runtime::vm::thread::Thread;

pub fn compile_dictionary_class(t: &mut Thread) {
    t.add_native_function("std_dictionary_constructor", dict_constructor);
    t.add_native_function("std_dictionary_get", dict_get);
    t.add_native_function("std_dictionary_set", dict_set);
    t.add_native_function("std_dictionary_length", dict_length);
    t.add_native_function("std_dictionary_remove", dict_remove);
    t.add_native_function("std_dictionary_clear", dict_clear);
    t.add_native_function("std_dictionary_keys", dict_keys);
    t.add_native_function("std_dictionary_values", dict_values);
    t.add_native_function("std_dictionary_contains_key", dict_contains_key);

    let mut class = generic_native_class!();
    class.insert(String::from(CONSTRUCTOR_NAME), Variant::NativeFunctionRef(String::from("std_dictionary_constructor")));
    class.insert(String::from("get"), Variant::NativeFunctionRef(String::from("std_dictionary_get")));
    class.insert(String::from("set"), Variant::NativeFunctionRef(String::from("std_dictionary_set")));
    class.insert(String::from("length"), Variant::NativeFunctionRef(String::from("std_dictionary_length")));
    class.insert(String::from("remove"), Variant::NativeFunctionRef(String::from("std_dictionary_remove")));
    class.insert(String::from("clear"), Variant::NativeFunctionRef(String::from("std_dictionary_clear")));
    class.insert(String::from("keys"), Variant::NativeFunctionRef(String::from("std_dictionary_keys")));
    class.insert(String::from("values"), Variant::NativeFunctionRef(String::from("std_dictionary_values")));
    class.insert(String::from("contains_key"), Variant::NativeFunctionRef(String::from("std_dictionary_contains_key")));
    t.add_global("Dictionary", Variant::Class(class));
}

fn dict_constructor(p: Vec<Variant>) -> Result<Option<Variant>, RuntimeError> {

    // this
    let Some(Variant::Object(this)) = p.get(PARAM_0) else {
        return Err(RuntimeError::InvalidNativeFunction(String::from("self")));
    };

    // if no parameters then return
    if p.len() > 1 {
        if let Variant::Map(initial_value) = p[PARAM_1].clone() {
            this.borrow_mut().insert(INTERNAL_CLASS_VALUE.to_string(), Variant::Map(initial_value));
        }
    }

    Ok(Some(Variant::Object(this.clone())))
}

fn dict_contains_key(p: Vec<Variant>) -> Result<Option<Variant>, RuntimeError> {

    // get key
    let Variant::String(key) = get_parameter(&p, PARAM_1)? else {
        return Err(RuntimeError::UnknownNativeParameterToken);
    };

    // get value
    let internal_value = get_object_value(&p)?;
    if internal_value.contains_key(&key) {
        return Ok(Some(Variant::Bool(true)));
    }

    Ok(Some(Variant::Bool(false)))
}

fn dict_keys(p: Vec<Variant>) -> Result<Option<Variant>, RuntimeError> {
    let internal_value = get_object_value(&p)?;

    let mut keys = Vec::new();
    for key in internal_value.keys() {
        keys.push(Variant::String(key.clone()));
    }

    Ok(Some(Variant::Array(keys)))
}

fn dict_values(p: Vec<Variant>) -> Result<Option<Variant>, RuntimeError> {
    let internal_value = get_object_value(&p)?;

    let mut values = Vec::new();
    for value in internal_value.values() {
        values.push(value.clone());
    }

    Ok(Some(Variant::Array(values)))
}

fn dict_get(p: Vec<Variant>) -> Result<Option<Variant>, RuntimeError> {

    // get key
    let Variant::String(key) = get_parameter(&p, PARAM_1)? else {
        return Err(RuntimeError::UnknownNativeParameterToken);
    };

    // get value
    let internal_value = get_object_value(&p)?;
    if let Some(v) = internal_value.get(&key) {
        return Ok(Some(v.clone()));
    }

    Err(RuntimeError::InvalidNativeFunction(String::from("Dictionary.get")))
}

fn dict_set(p: Vec<Variant>) -> Result<Option<Variant>, RuntimeError> {

    // get key
    let Variant::String(key) = get_parameter(&p, PARAM_1)? else {
        return Err(RuntimeError::UnknownNativeParameterToken);
    };

    // get value
    let value = get_parameter(&p, PARAM_2)?;

    // update
    let mut internal_value = get_object_value(&p)?;
    internal_value.insert(key, value);
    set_object_value(&p, internal_value);

    Ok(None)
}

fn dict_length(p: Vec<Variant>) -> Result<Option<Variant>, RuntimeError> {
    let internal_value = get_object_value(&p)?;
    Ok(Some(Variant::Integer(internal_value.len() as i64)))
}

fn dict_remove(p: Vec<Variant>) -> Result<Option<Variant>, RuntimeError> {
    let mut value = get_object_value(&p)?;

    let Variant::String(key) = p[PARAM_1].clone() else {
        return Err(RuntimeError::UnknownNativeParameterToken);
    };

    value.remove(&key);

    set_object_value(&p, value);

    Ok(None)
}

fn dict_clear(p: Vec<Variant>) -> Result<Option<Variant>, RuntimeError> {
    let mut value = get_object_value(&p)?;
    value.clear();
    set_object_value(&p, value);
    Ok(None)
}

fn get_parameter(p: &Vec<Variant>, i: usize) -> Result<Variant, RuntimeError> {
    if let Some(v) = p.get(i) {
        return Ok(v.clone());
    }
    Err(RuntimeError::UnknownNativeParameterToken)
}

fn get_object_value(p: &Vec<Variant>) -> Result<HashMap<String, Variant>, RuntimeError> {
    if let Variant::Object(obj) = p[PARAM_0].clone() {
        let borrowed = obj.borrow();
        if let Some(value) = borrowed.get(INTERNAL_CLASS_VALUE) {
            if let Variant::Map(map) = value {
                return Ok(map.clone());
            }
        }
    }
    Err(RuntimeError::InvalidNativeFunction(String::from("get object value")))
}

fn set_object_value(p: &Vec<Variant>, value: HashMap<String, Variant>) {
    if let Variant::Object(obj) = p[PARAM_0].clone() {
        let mut borrowed = obj.borrow_mut();
        borrowed.insert(String::from(INTERNAL_CLASS_VALUE), Variant::Map(value));
    } else {
        println!("Invalid object type");
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use super::*;

    macro_rules! construct_object {
        () => {
            {
                let mut test_map = HashMap::new();
                test_map.insert(String::from("a"), Variant::Integer(1));
                test_map.insert(String::from("b"), Variant::Integer(2));
                test_map.insert(String::from("c"), Variant::Integer(3));

                let mut class = HashMap::new();
                class.insert(String::from(INTERNAL_CLASS_VALUE), Variant::Map(test_map));

                Variant::Object(Rc::new(RefCell::new(class)))
            }
        };
    }

    #[test]
    fn test_constructor() {
        let input = vec![
            construct_object!(),
        ];

        match dict_constructor(input) {
            Ok(Some(Variant::Object(obj))) => {
                let borrowed = obj.borrow();
                if let Some(value) = borrowed.get(INTERNAL_CLASS_VALUE) {
                    if let Variant::Map(map) = value {
                        assert_eq!(map.len(), 3);
                    } else {
                        assert!(false, "value not a map");
                    }
                } else {
                    assert!(false, "internal value not found");
                }
            }
            _ => assert!(false, "constructor failed")
        }
    }

    #[test]
    fn test_dict_get() {
        let input = vec![
            construct_object!(),
            Variant::String(String::from("a")),
        ];

        match dict_get(input) {
            Ok(Some(Variant::Integer(1))) => assert!(true),
            _ => assert!(false, "get failed")
        }
    }

    #[test]
    fn test_dict_set_and_get() {
        let test_key = Variant::String(String::from("d"));
        let test_value = Variant::Integer(10);
        let class = construct_object!();
        let input = vec![
            class.clone(),
            test_key.clone(),
            test_value.clone(),
        ];

        // set value
        match dict_set(input) {
            Ok(None) => assert!(true),
            _ => assert!(false, "set failed")
        }

        // get value
        match dict_get(vec![class.clone(), test_key.clone()]) {
            Ok(Some(Variant::Integer(i))) => assert_eq!(i, 10),
            _ => assert!(false, "get failed")
        }
    }

    #[test]
    fn test_length() {
        let input = vec![construct_object!()];
        assert_eq!(dict_length(input), Ok(Some(Variant::Integer(3))));
    }

    #[test]
    fn test_dict_clear() {
        let input = vec![
            construct_object!(),
        ];

        dict_clear(input.clone());

        assert_eq!(dict_length(input.clone()), Ok(Some(Variant::Integer(0))));
    }

    #[test]
    fn test_dict_keys() {
        let input = vec![
            construct_object!(),
        ];

        match dict_keys(input) {
            Ok(Some(Variant::Array(keys))) => {
                assert_eq!(keys.len(), 3);

                // assert that the keys are in the array in any order
                assert!(keys.contains(&Variant::String(String::from("a"))), "does not contain a");
                assert!(keys.contains(&Variant::String(String::from("b"))), "does not contain b");
                assert!(keys.contains(&Variant::String(String::from("c"))), "does not contain c");
            }
            _ => assert!(false, "keys failed")
        }
    }

    #[test]
    fn test_dict_values() {
        let input = vec![
            construct_object!(),
        ];

        match dict_values(input) {
            Ok(Some(Variant::Array(values))) => {
                assert_eq!(values.len(), 3);

                // assert that the keys are in the array in any order
                assert!(values.contains(&Variant::Integer(1)), "does not contain 1");
                assert!(values.contains(&Variant::Integer(2)), "does not contain 2");
                assert!(values.contains(&Variant::Integer(3)), "does not contain 3");
            }
            _ => assert!(false, "values failed")
        }
    }

    #[test]
    fn test_dict_contains_key() {
        let input = vec![
            construct_object!(),
            Variant::String(String::from("a")),
        ];

        match dict_contains_key(input) {
            Ok(Some(Variant::Bool(true))) => assert!(true),
            _ => assert!(false, "contains_key failed")
        }
    }
}