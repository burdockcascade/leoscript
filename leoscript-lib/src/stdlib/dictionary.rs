use std::collections::HashMap;

use crate::common::variant::Variant;
use crate::{generic_native_class,  script_native_function_error};
use crate::common::error::{NativeFunctionError, ScriptError};
use crate::compiler::script::CONSTRUCTOR_NAME;
use crate::stdlib::{PARAM_1, PARAM_2, PARAM_0, INTERNAL_CLASS_VALUE};

pub fn compile_dictionary_class() -> Variant {

    let mut class = generic_native_class!();

    // new
    class.insert(String::from(CONSTRUCTOR_NAME), Variant::NativeFunction(|p| {

        // this
        let Some(Variant::Object(this)) = p.get(PARAM_0) else {
            return script_native_function_error!(NativeFunctionError::InvalidSelf)
        };

        // if no parameters then return
        if p.len() > 1 {
            if let Variant::Map(initial_value) = p[PARAM_1].clone() {
                this.borrow_mut().insert(INTERNAL_CLASS_VALUE.to_string(), Variant::Map(initial_value));
            }
        }

        Ok(Some(Variant::Object(this.clone())))

    }));

    // get (self, key)
    class.insert(String::from("get"), Variant::NativeFunction(|p| {

        // get key
        let Variant::String(key) = get_parameter(&p, PARAM_1)? else {
            return script_native_function_error!(NativeFunctionError::UnknownParameterToken)
        };

        // get value
        let internal_value = get_object_value(&p)?;
        if let Some(v) = internal_value.get(&key) {
            return Ok(Some(v.clone()));
        }

        script_native_function_error!(NativeFunctionError::InvalidNativeFunction(String::from("Dictionary.get")))
    }));

    // set (self, key, value)
    class.insert(String::from("set"), Variant::NativeFunction(|p| {

        // key
        let Variant::String(key) = get_parameter(&p, PARAM_1)? else {
            return script_native_function_error!(NativeFunctionError::UnknownParameterToken)
        };

        // value
        let value = get_parameter(&p, PARAM_2)?;

        // update
        let mut internal_value = get_object_value(&p)?;
        internal_value.insert(key, value);
        set_object_value(&p, internal_value);

        Ok(None)
    }));

    // length (self)
    class.insert(String::from("length"), Variant::NativeFunction(|p| {
        let value = get_object_value(&p)?;
        Ok(Some(Variant::Integer(value.len() as i32)))
    }));

    // remove (self, key)
    class.insert(String::from("remove"), Variant::NativeFunction(|p| {

        let mut value = get_object_value(&p)?;

        let Variant::String(key) = p[PARAM_1].clone() else {
            return script_native_function_error!(NativeFunctionError::UnknownParameterToken)
        };

        value.remove(&key);

        set_object_value(&p, value);

        Ok(None)
    }));

    // clear (self)
    class.insert(String::from("clear"), Variant::NativeFunction(|p| {

        let mut value = get_object_value(&p)?;

        value.clear();
        set_object_value(&p, value);
        Ok(None)
    }));

    Variant::Class(class)

}

fn get_object_value(p: &Vec<Variant>) -> Result<HashMap<String, Variant>, ScriptError> {
    if let Variant::Object(obj) =  p[PARAM_0].clone() {
        let borrowed = obj.borrow();
        if let Some(value) = borrowed.get(INTERNAL_CLASS_VALUE) {
            if let Variant::Map(map) = value {
                return Ok(map.clone());
            }
        }
    }
    script_native_function_error!(NativeFunctionError::InvalidInternalValue)
}

fn set_object_value(p: &Vec<Variant>, value: HashMap<String, Variant>) {
    if let Variant::Object(obj) =  p[PARAM_0].clone() {
        let mut borrowed = obj.borrow_mut();
        borrowed.insert(String::from(INTERNAL_CLASS_VALUE), Variant::Map(value));
    }
}

fn get_parameter(p: &Vec<Variant>, i: usize) -> Result<Variant, ScriptError> {
    if let Some(v) = p.get(i) {
        return Ok(v.clone());
    }
    script_native_function_error!(NativeFunctionError::UnknownParameterToken)
}