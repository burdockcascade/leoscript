use std::collections::HashMap;

use crate::common::variant::Variant;
use crate::stdlib::{PARAM_2, PARAM_3, PARAM_4, PARAM_1, INTERNAL_CLASS_VALUE};

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

        let this = p[PARAM_1].clone();

        let Some(value) = get_object_value(&this) else {
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

        let this = p[PARAM_1].clone();

        let Some(mut value) = get_object_value(&this) else {
            return None;
        };

        let Variant::String(key) = p[PARAM_2].clone() else {
            return None;
        };

        value.insert(key, p[PARAM_3].clone());

        set_object_value(&this, value);

        None
    }));

    Variant::Class(dict_class)
}

fn get_object_value(this: &Variant) -> Option<HashMap<String, Variant>> {

    let Variant::Object(obj) = this else {
        return None;
    };

    let borrowed = obj.borrow();
    match borrowed.get(INTERNAL_CLASS_VALUE) {
        Some(Variant::Map(map)) => Some(map.clone()),
        _ => None,
    }
}

fn set_object_value(this: &Variant, value: HashMap<String, Variant>) -> Option<()> {

    let Variant::Object(obj) = this else {
        return None;
    };

    let mut borrowed = obj.borrow_mut();
    borrowed.insert(String::from(INTERNAL_CLASS_VALUE), Variant::Map(value));
    return None;
}