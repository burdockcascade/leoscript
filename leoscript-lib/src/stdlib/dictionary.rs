use std::collections::HashMap;

use crate::common::variant::Variant;
use crate::stdlib::{PARAM_1, PARAM_2, PARAM_3, PARAM_SELF};

pub fn compile_dictionary_class() -> Variant {

    let mut dict_class = HashMap::new();

    dict_class.insert(String::from("_value"), Variant::Map(HashMap::new()));

    // new
    dict_class.insert(String::from("constructor"), Variant::NativeFunction(|p| {
        if let Variant::Map(d) = p[PARAM_SELF].clone() {
            return Some(Variant::Map(d));
        }
        None
    }));

    // set
    dict_class.insert(String::from("set"), Variant::NativeFunction(|p| {
        if let Variant::Map(mut d) = p[PARAM_1].clone() {
            d.insert(p[PARAM_2].to_string(), p[PARAM_3].clone());
            return Some(Variant::Map(d));
        }
        None
    }));

    // get
    dict_class.insert(String::from("get"), Variant::NativeFunction(|p| {
        if let Variant::Map(d) = p[PARAM_1].clone() {
            if let Some(v) = d.get(&p[PARAM_2].to_string()) {
                return Some(v.clone());
            }
        }
        None
    }));

    // remove
    dict_class.insert(String::from("remove"), Variant::NativeFunction(|p| {
        if let Variant::Map(mut d) = p[PARAM_1].clone() {
            d.remove(&p[PARAM_2].to_string());
            return Some(Variant::Map(d));
        }
        None
    }));

    // keys
    dict_class.insert(String::from("keys"), Variant::NativeFunction(|p| {
        if let Variant::Map(d) = p[PARAM_1].clone() {
            let mut keys = Vec::new();
            for k in d.keys() {
                keys.push(Variant::String(k.clone()));
            }
            return Some(Variant::Array(keys));
        }
        None
    }));

    // values
    dict_class.insert(String::from("values"), Variant::NativeFunction(|p| {
        if let Variant::Map(d) = p[PARAM_1].clone() {
            let mut values = Vec::new();
            for v in d.values() {
                values.push(v.clone());
            }
            return Some(Variant::Array(values));
        }
        None
    }));

    Variant::Class(dict_class)
}