use std::collections::HashMap;

use crate::common::error::ScriptError;
use crate::common::instruction::Instruction;
use crate::common::program::Program;
use crate::common::variant::Variant;
use crate::compiler::class::compile_class;
use crate::compiler::function::Function;
use crate::compiler::module::compile_module;
use crate::compiler::parser::parse_script;
use crate::compiler::r#enum::compile_enum;
use crate::compiler::token::Token;

pub const CONSTRUCTOR_NAME: &str = "constructor";
pub const SELF_CONSTANT: &str = "self";

pub struct FunctionGroup {
    pub structure: HashMap<String, Variant>,
    pub instructions: Vec<Instruction>,
}

pub struct FunctionSignature {
    pub name: String,
    pub input: Vec<Token>,
}

pub fn compile_script(source: &str) -> Result<Program, ScriptError> {

    let result = parse_script(source);

    // get tokens
    let tokens = match result {
        Ok((_, tokens)) => {
            tokens
        },
        Err(_e) => return Err(ScriptError::ParserError {
            line: 0,
            column: 0,
        }),
    };

    let mut p = Program::default();

    // collect function signatures
    let mut function_signatures = Vec::new();

    for token in tokens.clone() {
        match token {
            Token::Function { function_name, input, .. } => {
                function_signatures.push(FunctionSignature {
                    name: function_name.to_string(),
                    input,
                });
            },
            _ => {}
        }
    }

    // compile script
    for token in tokens.clone() {
        match token {
            Token::Function { position, function_name, input, body, .. } => {

                let func = Function::new(position, function_name.to_string(), input, body)?;
                p.globals.insert(function_name.to_string(), Variant::FunctionPointer(p.instructions.len()));
                p.instructions.append(&mut func.instructions.clone());

            },
            Token::Module { position, module_name, body, .. } => {
                let class_name_as_string = module_name.to_string();
                let mod_struct = compile_module(position, module_name, body, p.instructions.len())?;

                p.globals.insert(class_name_as_string, Variant::Module(mod_struct.structure));
                p.instructions.append(&mut mod_struct.instructions.clone());
            }
            Token::Class {position, class_name, body, .. } => {

                let class_name_as_string = class_name.to_string();
                let class_struct = compile_class(position, class_name, body, p.instructions.len())?;

                p.globals.insert(class_name_as_string, Variant::Class(class_struct.structure));
                p.instructions.append(&mut class_struct.instructions.clone());

            },
            Token::Enum { position, name, items } => {
                let enum_def = compile_enum(position, name.clone(), items)?;
                p.globals.insert(name, enum_def);
            }
            _ => {}
        }
    }

    Ok(p)

}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn complex_script() {

        // let _ = TermLogger::init(LevelFilter::Trace, Config::default(), TerminalMode::Mixed, ColorChoice::Auto);

        let source = r#"
            class Person

                var name
                var age
                var gender

                constructor(name, age, gender)
                    self.name = name
                    self.age = age
                    self.gender = gender
                end

                function get_name()
                    return self.name
                end

                function get_age()
                    return self.age
                end

            end

            class Employee

                var person
                var salary

                constructor(person, salary)
                    self.person = person
                    self.salary = salary
                end

                function get_person()
                    return self.person
                end

                function get_salary()
                    return self.salary
                end

            end

            Class Console

                static function log(message)
                    print(message)
                end

            end

            enum Gender
                Male
                Female
                Other
            end

            module Math

                function max(a, b)
                    if a > b then
                        return a
                    end
                    return b
                end

                function min(a, b)
                    if a < b then
                        return a
                    end
                    return b
                end

                class Vector2

                    var x as Integer
                    var y as Integer

                    constructor(x, y)
                        self.x = x
                        self.y = y
                    end

                    function add(other)
                        return new Vector2(self.x + other.x, self.y + other.y)
                    end

                    function sub(other)
                        return new Vector2(self.x - other.x, self.y - other.y)
                    end

                    function mul(other)
                        return new Vector2(self.x * other.x, self.y * other.y)
                    end

                    function div(other)
                        return new Vector2(self.x / other.x, self.y / other.y)
                    end

                    function dot(other)
                        return self.x * other.x + self.y * other.y
                    end

                    function cross(other)
                        return self.x * other.y - self.y * other.x
                    end

                    function length()
                        return Math.sqrt(self.x * self.x + self.y * self.y)
                    end

                    function normalize()
                        var l = self.length()
                        return new Vector2(self.x / l, self.y / l)
                    end

                end

            end

            function main()

                var company = new Company("My Company")

                var v1 = new Math.Vector2(1, 2)

            end
        "#;

        // assert compile script returns ok
        assert!(compile_script(source).is_ok());

        let program = compile_script(source).unwrap();

        // assert program has 4 globals
        assert_eq!(program.globals.len(), 6);

        assert!(program.globals.contains_key("Person"));
        let Variant::Class(person) = program.globals.get("Person").unwrap() else { panic!("Expected Person to be a class"); };
        assert!(person.contains_key("name"));
        assert!(person.contains_key("age"));
        assert!(person.contains_key("gender"));

        assert!(program.globals.contains_key("Employee"));
        let Variant::Class(employee) = program.globals.get("Employee").unwrap() else { panic!("Expected Employee to be a class"); };
        assert!(employee.contains_key("person"));
        assert!(employee.contains_key("salary"));

        assert!(program.globals.contains_key("Gender"));
        let Variant::Enum(gender) = program.globals.get("Gender").unwrap() else { panic!("Expected Gender to be a enum"); };
        assert!(gender.contains_key("Male"));
        assert!(gender.contains_key("Female"));
        assert!(gender.contains_key("Other"));

    }

    #[test]
    fn test_class_construct() {

        let script = r#"
            function main()
                var z = new Vector2(10, 20)
            end
        "#;

        let program = compile_script(script).unwrap();

    }

    #[test]
    fn test_class_construct_from_module() {

        let script = r#"
            function main()
                var p = new Hotel.Person("John", 30)
            end

            module Hotel

                class Person

                    var name
                    var age

                    constructor(name, age)
                        self.name = name
                        self.age = age
                    end

                    function get_name()
                        return self.name
                    end

                    function get_age()
                        return self.age
                    end

                end

            end
        "#;

        let program = compile_script(script).unwrap();

    }
}