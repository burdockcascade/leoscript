use std::collections::HashMap;
use std::env::current_dir;
use std::fs;
use std::path::Path;
use std::time::Duration;

use crate::common::error::{ParseError, ScriptError, SystemError};
use crate::common::instruction::Instruction;
use crate::common::program::Program;
use crate::common::variant::Variant;
use crate::compiler::class::compile_class;
use crate::compiler::function::Function;
use crate::compiler::module::compile_module;
use crate::compiler::parser::parse_script;
use crate::compiler::r#enum::compile_enum;
use crate::compiler::token::Token;
use crate::{script_compile_error, script_compile_warning, script_parse_error, script_system_error};
use crate::common::error::CompilerError::{InvalidImportPath, UnableToImportFile};
use crate::common::warning::{CompilerWarning, ScriptWarning};

pub const CONSTRUCTOR_NAME: &str = "constructor";
pub const SELF_CONSTANT: &str = "self";

pub struct FunctionGroup {
    pub structure: HashMap<String, Variant>,
    pub instructions: Vec<Instruction>,
}

struct Script {
    pub instructions: Vec<Instruction>,
    pub globals: HashMap<String, Variant>,
    pub warnings: Vec<ScriptWarning>,
}

pub struct CompilerResult {
    pub program: Program,
    pub compile_time: Duration,
    pub warnings: Vec<ScriptWarning>,
}

pub fn compile_program(source: &str) -> Result<CompilerResult, ScriptError> {

    // start timer
    let start_compiler = std::time::Instant::now();

    // compile master script
    let script = compile_script(source, 0)?;

    // end timer
    let end_compiler = std::time::Instant::now();

    // return script result
    Ok(CompilerResult {
        program: Program {
            instructions: script.instructions,
            globals: script.globals
        },
        compile_time: end_compiler - start_compiler,
        warnings: script.warnings,
    })

}

fn compile_script(source: &str, offset: usize) -> Result<Script, ScriptError> {

    let mut script = Script {
        instructions: Vec::new(),
        globals: HashMap::new(),
        warnings: Vec::new(),
    };

    // get tokens
    let tokens = match parse_script(source) {
        Ok((_, tokens)) => tokens,
        Err(_e) => return script_parse_error!(ParseError::UnableToParseTokens),
    };

    // compile imports
    for token in tokens.clone() {

        let local_offset = script.instructions.len() + offset;

        // set path separator based on local system
        let path_separator = match std::env::consts::OS {
            "windows" => "\\",
            _ => "/"
        };

        match token {
            Token::Import { position, source, .. } => {

                // get current directory
                let Ok(dir) = current_dir() else {
                    return script_system_error!(SystemError::UnableToGetLocalDirectory);
                };

                // get file path
                let mut filepath = dir.display().to_string() + path_separator;

                // convert list of identifiers to path
                for (i, identifier) in source.iter().enumerate() {
                    if let Token::Identifier { name, .. } = identifier {
                        filepath += name;
                        if i < source.len() - 1 {
                            filepath += path_separator;
                        }
                    }
                }

                let filename = filepath.clone() + ".leo";

                // check if file exists
                if Path::new(&filename).exists() {

                    // read file contents
                    let Ok(contents) = fs::read_to_string(filename.clone()) else {
                        return script_compile_error!(UnableToImportFile(filename.clone()), position);
                    };

                    // compile imported script
                    let mut imported_script = compile_script(&contents, local_offset)?;

                    if imported_script.globals.len() == 0 {
                        script.warnings.push(script_compile_warning!(CompilerWarning::NothingToImport, position))
                    }

                    // add imported script globals to script globals
                    for (key, value) in imported_script.globals.iter() {
                        script.globals.insert(key.to_string(), value.clone());
                    }

                    // add imported script instructions to script instructions
                    script.instructions.append(&mut imported_script.instructions);

                } else {
                    return script_compile_error!(InvalidImportPath(filename), position);
                }

            }
            _ => {}
        }
    }

    // compile script
    for token in tokens.clone() {

        let local_offset = script.instructions.len() + offset;

        match token {
            Token::Function { position, function_name, input, body, .. } => {

                let func = Function::new(position, function_name.to_string(), input, body)?;
                script.globals.insert(function_name.to_string(), Variant::FunctionPointer(local_offset));
                script.instructions.append(&mut func.instructions.clone());

            },
            Token::Module { position, module_name, body, .. } => {
                let class_name_as_string = module_name.to_string();
                let mod_struct = compile_module(position, module_name, body, local_offset)?;

                script.globals.insert(class_name_as_string, Variant::Module(mod_struct.structure));
                script.instructions.append(&mut mod_struct.instructions.clone());
            }
            Token::Class {position, class_name, body, .. } => {

                let class_name_as_string = class_name.to_string();
                let class_struct = compile_class(position, class_name, body, local_offset)?;

                script.globals.insert(class_name_as_string, Variant::Class(class_struct.structure));
                script.instructions.append(&mut class_struct.instructions.clone());

            },
            Token::Enum { position, name, items } => {
                let enum_def = compile_enum(position, name.clone(), items)?;
                script.globals.insert(name, enum_def);
            }
            _ => {}
        }
    }

    Ok(script)

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