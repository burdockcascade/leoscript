use leoscript::compiler::error::{CompilerError, CompilerErrorType};
use leoscript::runtime::ir::variant::Variant;
use leoscript::{run_script, ScriptError};
use leoscript::compiler::parser::token::TokenPosition;

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
    "#, None, ScriptError::CompilerError(
        CompilerError {
            error: CompilerErrorType::VariableAlreadyDeclared(String::from("a")),
            position: TokenPosition {
                line: 4,
                column: 14,
            },
        }
    ))
}

#[test]
fn variable_is_not_declared_error() {
    test_failure!(r#"
          function main()
             a = 5
             return a
         end
    "#, None, ScriptError::CompilerError(
        CompilerError {
            error: CompilerErrorType::VariableNotDeclared(String::from("a")),
            position: TokenPosition {
                line: 3,
                column: 14,
            },
        }
    ));
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

#[test]
fn pass_by_value() {
    test_success!(r#"
        function main()
            var x = 5
            var y = 10
            swap(x, y)
            return x == 5 and y == 10
        end

        function swap(a, b)
            var temp = a
            a = b
            b = temp
        end
    "#);
}
