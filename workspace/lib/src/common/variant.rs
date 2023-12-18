use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::ops::{Add, BitAnd, BitOr, Div, Mul, Not, Sub};
use std::rc::Rc;

use log::error;

use crate::common::counter::Counter;

// Value
#[derive(Clone, PartialEq, Debug)]
pub enum Variant {
    Any,

    // Primitive Types
    Null,
    Integer(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Array(Vec<Variant>),
    Map(HashMap<String, Variant>),
    Enum(HashMap<String, usize>),
    Module(HashMap<String, Variant>),
    Class(HashMap<String, Variant>),
    Object(Rc<RefCell<HashMap<String, Variant>>>),

    // References and Pointers
    FunctionRef(String),
    FunctionPointer(usize),

    // Native Functions
    NativeFunctionRef(String),

    // counter with start, step, end
    Iterator(Box<Counter>),

    Type(String),

    FramePointer(usize),
    ReturnPointer(usize),
}

#[derive(Clone, PartialEq, Debug)]
pub enum ValueType {
    Null,
    Any,
    Integer,
    Float,
    Bool,
    String,
    Array,
    Dictionary,
    Global(String),
}

impl ValueType {
    pub fn default_value(&self) -> Variant {
        match self {
            ValueType::Integer => Variant::Integer(0),
            ValueType::Float => Variant::Float(0.0),
            ValueType::Bool => Variant::Bool(false),
            ValueType::String => Variant::String("".to_string()),
            _ => Variant::Null
        }
    }
}

impl Display for Variant {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Variant::Null => write!(f, "null"),
            Variant::Integer(num) => write!(f, "{num}"),
            Variant::Float(num) => write!(f, "{num}"),
            Variant::Bool(b) => write!(f, "{b}"),
            Variant::String(string) => write!(f, "{string}"),
            Variant::Array(_val) => write!(f, "Array"),
            Variant::FunctionRef(i) => write!(f, "{i}"),
            _ => write!(f, "todo for {:?}", self),
        }
    }
}

impl BitOr for Variant {
    type Output = Variant;

    fn bitor(self, rhs: Variant) -> <Self as BitOr<Variant>>::Output {
        match (self, rhs) {
            (Variant::Bool(v1), Variant::Bool(v2)) => Variant::Bool(v1 | v2),
            _ => unreachable!("can not or values")
        }
    }
}

impl BitAnd for Variant {
    type Output = Variant;

    fn bitand(self, rhs: Variant) -> <Self as BitAnd<Variant>>::Output {
        match (self, rhs) {
            (Variant::Bool(v1), Variant::Bool(v2)) => Variant::Bool(v1 & v2),
            _ => unreachable!("can not or values")
        }
    }
}

// Value Comparison
impl PartialOrd for Variant {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        match (self, rhs) {
            (Variant::Integer(v1), Variant::Integer(v2)) => v1.partial_cmp(v2),
            (Variant::Float(v1), Variant::Float(v2)) => v1.partial_cmp(v2),
            _ => unreachable!("can not subtract values")
        }
    }
}

// Value Subtraction
impl Sub for Variant {
    type Output = Variant;

    fn sub(self, rhs: Variant) -> <Self as Sub<Variant>>::Output {
        match (self, rhs) {
            (Variant::Integer(v1), Variant::Integer(v2)) => Variant::Integer(v1 - v2),
            (Variant::Integer(v1), Variant::Float(v2)) => Variant::Float(v1 as f64 - v2),
            (Variant::Float(v1), Variant::Integer(v2)) => Variant::Float(v1 - v2 as f64),
            (Variant::Float(v1), Variant::Float(v2)) => Variant::Float(v1 - v2),
            _ => unreachable!("can not subtract values")
        }
    }
}


// Value Addition
impl Add for Variant {
    type Output = Variant;

    fn add(self, rhs: Variant) -> <Self as Add<Variant>>::Output {
        if Variant::Null == self || Variant::Null == rhs {
            panic!("Can not add null value")
        }

        match (self, rhs) {

            // add integers together
            (Variant::Integer(v1), Variant::Integer(v2)) => Variant::Integer(v1 + v2),
            (Variant::Integer(v1), Variant::Float(v2)) => Variant::Float(v1 as f64 + v2),
            (Variant::Integer(v1), Variant::String(v2)) => Variant::String(v1.to_string().add(v2.as_str())),

            // add floats together
            (Variant::Float(v1), Variant::Integer(v2)) => Variant::Float(v1 + v2 as f64),
            (Variant::Float(v1), Variant::Float(v2)) => Variant::Float(v1 + v2),

            // add strings together
            (Variant::String(v1), Variant::String(v2)) => Variant::String(v1.add(v2.as_str())),
            (Variant::String(v1), Variant::Bool(v2)) => Variant::String(v1.add(v2.to_string().as_str())),
            (Variant::String(v1), Variant::Integer(v2)) => Variant::String(v1.add(v2.to_string().as_str())),
            (Variant::String(v1), Variant::Float(v2)) => Variant::String(v1.add(v2.to_string().as_str())),

            // add arrays together
            (Variant::Array(mut v1), Variant::Array(v2)) => {
                v1.extend(v2);
                Variant::Array(v1)
            }

            // add booleans together but only true + true = true
            (Variant::Bool(v1), Variant::Bool(v2)) => Variant::Bool(v1 && v2),

            _ => unreachable!("can not add values")
        }
    }
}

// Value Multiplication
impl Mul for Variant {
    type Output = Variant;

    fn mul(self, rhs: Variant) -> <Self as Mul<Variant>>::Output {

        let lhs = self;

        match (lhs.clone(), rhs.clone()) {
            (Variant::Integer(v1), Variant::Integer(v2)) => Variant::Integer(v1 * v2),
            (Variant::Integer(v1), Variant::Float(v2)) => Variant::Float(v1 as f64 * v2),
            (Variant::Float(v1), Variant::Integer(v2)) => Variant::Float(v1 * v2 as f64),
            (Variant::Float(v1), Variant::Float(v2)) => Variant::Float(v1 * v2),
            _ => {
                error!("Multiplying {:?} * {:?}", lhs, rhs);
                unreachable!("can not multiply values")
            }
        }
    }
}

// Value Division
impl Div for Variant {
    type Output = Variant;

    fn div(self, rhs: Variant) -> <Self as Div<Variant>>::Output {
        match (self, rhs) {
            (Variant::Integer(v1), Variant::Integer(v2)) => Variant::Integer(v1 / v2),
            (Variant::Integer(v1), Variant::Float(v2)) => Variant::Float(v1 as f64 / v2),
            (Variant::Float(v1), Variant::Integer(v2)) => Variant::Float(v1 / v2 as f64),
            (Variant::Float(v1), Variant::Float(v2)) => Variant::Float(v1 / v2),
            _ => unreachable!("can not divide values")
        }
    }
}

// Value Negation
impl Not for Variant {
    type Output = Variant;

    fn not(self) -> Self::Output {
        match self {
            Variant::Bool(true) => Variant::Bool(false),
            Variant::Bool(false) => Variant::Bool(true),
            _ => Variant::Bool(false),
        }
    }
}

// power of
impl Variant {
    pub fn pow(&self, rhs: Variant) -> Variant {
        match (self, rhs) {
            (Variant::Integer(v1), Variant::Integer(v2)) => Variant::Integer(v1.pow(v2 as u32)),
            (Variant::Integer(v1), Variant::Float(v2)) => Variant::Float((*v1 as f64).powf(v2)),
            (Variant::Float(v1), Variant::Integer(v2)) => Variant::Float(v1.powf(v2 as f64)),
            (Variant::Float(v1), Variant::Float(v2)) => Variant::Float(v1.powf(v2)),
            _ => unreachable!("can not pow values")
        }
    }
}

#[cfg(test)]
mod test {
    use crate::common::variant::Variant;

    #[test]
    fn test_add() {

        // integers
        assert_eq!(Variant::Integer(2) + Variant::Integer(3), Variant::Integer(5));
        assert_eq!(Variant::Integer(2) + Variant::Float(3.3), Variant::Float(5.3));

        // floats
        assert_eq!(Variant::Float(2.2) + Variant::Float(3.3), Variant::Float(5.5));
        assert_eq!(Variant::Float(2.2) + Variant::Integer(3), Variant::Float(5.2));

        // strings
        assert_eq!(Variant::String(String::from("x = ")) + Variant::Integer(3), Variant::String(String::from("x = 3")));
        assert_eq!(Variant::String(String::from("x = ")) + Variant::Float(3.1), Variant::String(String::from("x = 3.1")));
        assert_eq!(Variant::String(String::from("x = ")) + Variant::Bool(true), Variant::String(String::from("x = true")));

        // true and false booleans should return false
        assert_eq!(Variant::Bool(true) + Variant::Bool(false), Variant::Bool(false));
        assert_eq!(Variant::Bool(false) + Variant::Bool(true), Variant::Bool(false));
        assert_eq!(Variant::Bool(false) + Variant::Bool(false), Variant::Bool(false));
        assert_eq!(Variant::Bool(true) + Variant::Bool(true), Variant::Bool(true));
    }

    #[test]
    fn test_sub() {
        assert_eq!(Variant::Integer(7) - Variant::Integer(3), Variant::Integer(4));
        assert_eq!(Variant::Integer(5) - Variant::Float(3.3), Variant::Float(1.7000000000000002));
        assert_eq!(Variant::Float(2.4) - Variant::Float(1.3), Variant::Float(1.0999999999999999));
        assert_eq!(Variant::Float(5.2) - Variant::Integer(3), Variant::Float(2.2));
    }

    #[test]
    fn test_mul() {
        assert_eq!(Variant::Integer(7) * Variant::Integer(3), Variant::Integer(21));
        assert_eq!(Variant::Integer(5) * Variant::Float(1.1), Variant::Float(5.5));
        assert_eq!(Variant::Float(2.4) * Variant::Float(1.3), Variant::Float(3.12));
        assert_eq!(Variant::Float(5.2) * Variant::Integer(3), Variant::Float(15.600000000000001));
    }

    #[test]
    fn test_div() {
        assert_eq!(Variant::Integer(21) / Variant::Integer(3), Variant::Integer(7));
        assert_eq!(Variant::Integer(22) / Variant::Float(1.1), Variant::Float(20.0));
        assert_eq!(Variant::Float(2.4) / Variant::Float(1.3), Variant::Float(1.846153846153846));
        assert_eq!(Variant::Float(5.2) / Variant::Integer(3), Variant::Float(1.7333333333333334));
    }

    #[test]
    fn test_eq() {
        assert_eq!(Variant::Integer(3) == Variant::Integer(3), true);
        assert_eq!(Variant::Integer(21) == Variant::Integer(3), false);
        assert_eq!(Variant::Float(2.0) == Variant::Integer(2), false);
        assert_eq!(Variant::Float(2.0) == Variant::Float(2.0), true);
        assert_eq!(Variant::Bool(true) == Variant::Bool(true), true);
        assert_eq!(Variant::Bool(false) != Variant::Bool(true), true);
        assert_eq!(Variant::String("hello world".parse().unwrap()) == Variant::String("hello world".parse().unwrap()), true);
        assert_eq!(Variant::String("hello world".parse().unwrap()) == Variant::String("goodbye world".parse().unwrap()), false);
    }

    #[test]
    fn test_cmp() {
        assert_eq!(Variant::Integer(6) > Variant::Integer(3), true);
        assert_eq!(Variant::Integer(6) < Variant::Integer(30), true);
        assert_eq!(Variant::Float(6.1) > Variant::Float(3.5), true);
    }
}