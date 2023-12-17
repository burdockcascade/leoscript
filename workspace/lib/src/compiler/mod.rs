use std::time::Duration;
use nom_locate::LocatedSpan;
use crate::common::error::ScriptError;
use crate::common::program::Program;
use crate::common::warning::ScriptWarning;
use crate::compiler::script::compile_script;

mod token;
mod parser;
pub mod script;
mod class;
mod function;
mod variable;
mod module;
mod r#enum;

type Span<'a> = LocatedSpan<&'a str>;

pub struct CompilerResult {
    pub program: Program,
    pub compile_time: Duration,
    pub parser_time: Duration,
    pub warnings: Vec<ScriptWarning>,
}

pub fn compile_program(source: &str) -> Result<CompilerResult, ScriptError> {

    // compile master script
    let script = compile_script(source, 0)?;

    // return script result
    Ok(CompilerResult {
        program: Program {
            instructions: script.instructions,
            globals: script.globals
        },
        compile_time: script.compiler_time,
        parser_time: script.parser_time,
        warnings: script.warnings,
    })

}

#[cfg(test)]
mod test {
    use crate::common::variant::Variant;
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
        assert!(compile_program(source).is_ok());

        let compiler_result = compile_program(source).unwrap();
        let program = compiler_result.program;

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

            class Vector2

                var x
                var y

                constructor(x, y)
                    self.x = x
                    self.y = y
                end

            end

        "#;

        let result = compile_program(script).unwrap();
        let program = result.program;

        // fixme
        assert!(program.globals.contains_key("Vector2"));

    }

    #[test]
    fn test_class_construct_from_module() {

        let script = r#"
            function main()
                var p = new Company.Person("John", 30)
            end

            module Company

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

        let program = compile_program(script).unwrap();

        // fixme

    }
}