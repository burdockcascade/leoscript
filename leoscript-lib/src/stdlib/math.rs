use std::collections::HashMap;

use crate::common::variant::Variant;
use crate::stdlib::{PARAM_1, PARAM_2};

pub fn compile_math_module() -> Variant {

    let mut mhash = HashMap::new();

    // min
    mhash.insert(String::from("min"), Variant::NativeFunction(|p| {
        if let Variant::Integer(i) = p[PARAM_1] {
            if let Variant::Integer(j) = p[PARAM_2] {
                return Some(Variant::Integer(i.min(j)));
            }
        }
        None
    }));

    // max
    mhash.insert(String::from("max"), Variant::NativeFunction(|p| {
        if let Variant::Integer(i) = p[PARAM_1] {
            if let Variant::Integer(j) = p[PARAM_2] {
                return Some(Variant::Integer(i.max(j)));
            }
        }
        None
    }));

    // sqrt
    mhash.insert(String::from("sqrt"), Variant::NativeFunction(|p| {
        if let Variant::Integer(i) = p[PARAM_1] {
            return Some(Variant::Float((i as f32).sqrt()));
        }
        None
    }));

    // abs
    mhash.insert(String::from("abs"), Variant::NativeFunction(|p| {
        if let Variant::Integer(i) = p[PARAM_1] {
            return Some(Variant::Integer(i.abs()));
        }
        None
    }));

    Variant::Module(mhash)

}