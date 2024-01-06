use crate::runtime::error::RuntimeError;
use crate::runtime::ir::variant::Variant;

pub trait NativeClass: PartialEq {

    fn constructor(&self, params: Vec<Variant>) -> Result<Self, RuntimeError> where Self: Sized;

    fn call(&self, name: &str, params: Vec<Variant>) -> Result<Option<Box<Variant>>, RuntimeError>;

}

#[derive(Clone, PartialEq, Debug)]
pub struct MyClass {
    value: Vec<Variant>
}

impl MyClass {

    pub fn new() -> Self {
        MyClass {
            value: Vec::new()
        }
    }

}

impl PartialEq for MyClass {
    fn eq(&self, other: &Self) -> bool {
        false
    }
}

impl NativeClass for MyClass {

    fn constructor(&self, params: Vec<Variant>) -> Result<Self, RuntimeError> {
        Ok(MyClass {
            value: params
        })
    }

    fn call(&self, name: &str, params: Vec<Variant>) -> Result<Option<Box<Variant>>, RuntimeError> {

        let Some(Variant::Integer(index)) = params.get(0) else {
            panic!("Invalid parameter type");
        };

        match name {
            "get_value" => Ok(Some(Box::from(self.value[*index as usize].clone()))),
            _ => panic!("Unknown method")
        }
    }

}

#[cfg(test)]
mod test {
    use crate::runtime::ir::variant::Variant;
    use crate::runtime::stdlib::nclass::{MyClass, NativeClass};

    #[test]
    fn test() {
        let my_class = MyClass::new();
        let my_class = my_class.constructor(vec![Variant::Integer(42)]).unwrap();
        let result = my_class.call("get_value", vec![Variant::Integer(0)]).unwrap();
        assert_eq!(result, Some(Variant::Integer(42)));
    }

}
