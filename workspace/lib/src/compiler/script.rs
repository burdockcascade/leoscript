use std::collections::HashMap;
use std::env::current_dir;
use std::fs;
use std::path::Path;
use std::time::Duration;

use crate::{script_compile_error, script_compile_warning, script_system_error};
use crate::common::error::{ScriptError, SystemError};
use crate::common::error::CompilerError::{InvalidImportPath, UnableToImportFile};
use crate::common::instruction::Instruction;
use crate::common::variant::Variant;
use crate::common::warning::{CompilerWarning, ScriptWarning};
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

pub struct Script {
    pub instructions: Vec<Instruction>,
    pub globals: HashMap<String, Variant>,
    pub compiler_time: Duration,
    pub parser_time: Duration,
    pub warnings: Vec<ScriptWarning>,
}

pub(crate) fn compile_script(source: &str, offset: usize) -> Result<Script, ScriptError> {

    let mut script = Script {
        instructions: Vec::new(),
        globals: HashMap::new(),
        compiler_time: Default::default(),
        parser_time: Default::default(),
        warnings: Vec::new(),
    };



    // get tokens
    let parser_result = parse_script(source)?;

    script.parser_time = parser_result.parser_time;

    let start_compiler_timer = std::time::Instant::now();

    // compile imports
    for token in parser_result.tokens.clone() {

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

                    // update parser timer
                    script.parser_time += imported_script.parser_time;

                    // update compiler timer
                    script.compiler_time += imported_script.compiler_time;

                } else {
                    return script_compile_error!(InvalidImportPath(filename), position);
                }

            }
            _ => {}
        }
    }

    // compile script
    for token in parser_result.tokens.clone() {

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

    script.compiler_time += start_compiler_timer.elapsed();

    Ok(script)

}