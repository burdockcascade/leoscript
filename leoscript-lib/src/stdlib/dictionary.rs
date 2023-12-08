use std::collections::HashMap;

use crate::common::variant::Variant;
use crate::stdlib::{PARAM_2, PARAM_3, PARAM_4, PARAM_1, INTERNAL_CLASS_VALUE};

macro_rules! get_object_value {
    ($p:ident) => {
        if let Variant::Object(obj) =  $p[PARAM_1].clone() {
            let borrowed = obj.borrow();
            if let Variant::Map(value) = borrowed.get(INTERNAL_CLASS_VALUE).unwrap() {
                Some(value.clone())
            } else {
                None
            }
        } else {
            None
        }
    };
}

macro_rules! set_object_value {
    ($p:ident, $value:ident) => {
        if let Variant::Object(obj) =  $p[PARAM_1].clone() {
            let mut borrowed = obj.borrow_mut();
            borrowed.insert(String::from(INTERNAL_CLASS_VALUE), Variant::Map($value));
        }
    };
}

pub fn compile_dictionary_class() -> Variant {

    let mut dict_class = HashMap::new();

    dict_class.insert(String::from("_value"), Variant::Map(HashMap::new()));

    // new
    dict_class.insert(String::from("constructor"), Variant::NativeFunction(|p| {
        if let Variant::Object(d) = p[PARAM_1].clone() {
            return Some(p[PARAM_1].clone());
        }
        None
    }));

    // get
    dict_class.insert(String::from("get"), Variant::NativeFunction(|p| {

        let Some(value) = get_object_value!(p) else {
            return None;
        };

        let Variant::String(key) = p[PARAM_2].clone() else {
            return None;
        };

        if let Some(v) = value.get(&key) {
            return Some(v.clone());
        }

        None
    }));

    // set (self, key, value)
    dict_class.insert(String::from("set"), Variant::NativeFunction(|p| {

        let Some(mut value) = get_object_value!(p) else {
            return None;
        };

        let Variant::String(key) = p[PARAM_2].clone() else {
            return None;
        };

        value.insert(key, p[PARAM_3].clone());

        set_object_value!(p, value);

        None
    }));

    Variant::Class(dict_class)
}

fn set_object_value(this: &Variant, value: HashMap<String, Variant>) -> Option<()> {

    let Variant::Object(obj) = this else {
        return None;
    };

    let mut borrowed = obj.borrow_mut();
    borrowed.insert(String::from(INTERNAL_CLASS_VALUE), Variant::Map(value));
    return None;
}