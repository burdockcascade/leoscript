use leoscript_lib::common::error::{CompilerError, ScriptError};
use leoscript_lib::common::variant::Variant;
use leoscript_lib::run_script;

mod common;

#[test]
fn string_set() {
    test_success!(r#"
         function main()
             var name = "peter"
             return name == "peter"
         end
    "#);
}

#[test]
fn integers() {
    test_success!(r#"
         function main()
            var number1 = 1
            var number2 = 2
            var number3 = 3
            return number1 == 1
        end
    "#);
}

#[test]
fn floats() {
    test_success!(r#"
         function main()
            var f1 as float = 3.14
            var f2 as float = 2.7
            return f2 == 2.7
        end
    "#);
}

#[test]
fn booleans() {
    test_success!(r#"
         function main()
            var v1 as Boolean = true
            var v2 as Boolean = false
            return v1
        end
    "#);
}

#[test]
fn multi_value() {
    test_success!(r#"
        function main()
            var v1 = 1
            v1 = "fish"
            v1 = true
            return v1
        end
    "#);
}

#[test]
fn no_value() {
    test_success!(r#"
         function main()
             var a
             return a == null
         end
    "#);
}

#[test]
fn variable_declared_already_error() {
    test_failure!(r#"
          function main()
             var a
             var a
             return a
         end
    "#, None, CompilerError::VariableAlreadyDeclared(String::from("a")), 4, 14);
}

#[test]
fn variable_is_not_declared_error() {
    test_failure!(r#"
          function main()
             a = 5
             return a
         end
    "#, None, CompilerError::VariableNotDeclared(String::from("a")), 3, 14);
}

#[test]
fn enums() {
    test_success!(r#"
        enum Color
            Red
            Green
            Blue
        end
        
        function main()
            var x as Color = Color.Red
            var y as Color = Color.Green
            return x != y
        end
    "#);
}