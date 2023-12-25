use std::collections::HashMap;
use std::env::current_dir;
use std::fs;
use std::path::Path;
use std::time::Duration;
use leoscript_runtime::ir::instruction::Instruction;
use leoscript_runtime::ir::variant::Variant;

use crate::codegen::class::generate_class;
use crate::codegen::function::Function;
use crate::codegen::module::generate_module;
use crate::codegen::r#enum::generate_enum;
use crate::error::{CompilerError, CompilerErrorType};
use crate::parser::script::parse_script;
use crate::parser::token::Token;
use crate::warning::{CompilerWarning, CompilerWarningType};

const FILE_EXTENSION: &str = ".leo";
pub const CONSTRUCTOR_NAME: &str = "constructor";
pub const SELF_CONSTANT: &str = "self";

pub struct CodeStructure {
    pub structure: HashMap<String, Variant>,
    pub instructions: Vec<Instruction>,
}

impl Default for CodeStructure {
    fn default() -> Self {
        CodeStructure {
            structure: HashMap::default(),
            instructions: Vec::default(),
        }
    }
}

pub struct Script {
    pub instructions: Vec<Instruction>,
    pub globals: HashMap<String, Variant>,
    pub compiler_time: Duration,
    pub parser_time: Duration,
    pub imports: Vec<String>,
    pub warnings: Vec<CompilerWarning>,
}

impl Default for Script {
    fn default() -> Self {
        Script {
            instructions: Default::default(),
            globals: Default::default(),
            compiler_time: Default::default(),
            parser_time: Default::default(),
            imports: Default::default(),
            warnings: Default::default(),
        }
    }
}

pub fn generate_script(source: &str, offset: usize) -> Result<Script, CompilerError> {
    let mut script = Script::default();

    // get tokens
    let parser_result = match parse_script(source) {
        Ok(r) => r,
        Err(e) => return Err(CompilerError {
            error: CompilerErrorType::ParseError,
            position: Default::default(),
        })
    };

    // update parser timer
    script.parser_time = parser_result.parser_time;

    // set local offset
    let local_offset = script.instructions.len() + offset;

    // set path separator based on local system
    let path_separator = match std::env::consts::OS {
        "windows" => "\\",
        _ => "/"
    };

    // compile imports
    for token in parser_result.tokens.clone() {
        match token {
            Token::Import { position, source, .. } => {

                // get current directory
                let Ok(dir) = current_dir() else {
                    return Err(CompilerError {
                        error: CompilerErrorType::UnableToGetWorkingDirectory,
                        position,
                    })
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

                let filename = filepath.clone() + FILE_EXTENSION;

                // check if file exists
                if !Path::new(&filename).exists() {
                    return Err(CompilerError {
                        error: CompilerErrorType::InvalidImportPath(filename.clone()),
                        position,
                    });
                }

                // read file contents
                let Ok(contents) = fs::read_to_string(filename.clone()) else {
                    return Err(CompilerError {
                        error: CompilerErrorType::InvalidImportPath(filename.clone()),
                        position,
                    });
                };

                // add imported file to source files
                script.imports.push(filename.clone());

                // compile imported script
                let mut imported_script = generate_script(&contents, local_offset)?;

                // warn if nothing was imported
                if imported_script.globals.len() == 0 {
                    script.warnings.push(CompilerWarning {
                        warning: CompilerWarningType::ImportFileEmpty(filename.clone()),
                        position,
                    });
                }

                // add imported script globals to script globals
                for (key, value) in imported_script.globals.iter() {
                    script.globals.insert(key.to_string(), value.clone());
                }

                // add imported script instructions to script instructions
                script.instructions.append(&mut imported_script.instructions);

                // update parser timer
                script.parser_time += imported_script.parser_time;

                // update compiler3 timer
                script.compiler_time += imported_script.compiler_time;

                // update warnings
                script.warnings.append(&mut imported_script.warnings);

                // update source files
                script.imports.append(&mut imported_script.imports);
            }
            _ => {}
        }
    }

    let start_compiler_timer = std::time::Instant::now();

    // compile script
    for token in parser_result.tokens.clone() {
        let local_offset = script.instructions.len() + offset;

        match token {
            Token::Function { position, function_name, input, body, .. } => {
                let func = Function::new(position, function_name.to_string(), input, body)?;
                script.globals.insert(function_name.to_string(), Variant::FunctionPointer(local_offset));
                script.instructions.append(&mut func.instructions.clone());
            }
            Token::Module { position, module_name, body, .. } => {
                let class_name_as_string = module_name.to_string();
                let mod_struct = generate_module(position, module_name, body, local_offset)?;

                script.globals.insert(class_name_as_string, Variant::Module(mod_struct.structure));
                script.instructions.append(&mut mod_struct.instructions.clone());
            }
            Token::Class { position, class_name, body, .. } => {
                let class_name_as_string = class_name.to_string();
                let class_struct = generate_class(position, class_name, body, local_offset)?;

                script.globals.insert(class_name_as_string, Variant::Class(class_struct.structure));
                script.instructions.append(&mut class_struct.instructions.clone());
            }
            Token::Enum { position, name, items } => {
                let enum_def = generate_enum(position, name.clone(), items)?;
                script.globals.insert(name, enum_def);
            }
            _ => {}
        }
    }

    script.compiler_time += start_compiler_timer.elapsed();

    Ok(script)
}