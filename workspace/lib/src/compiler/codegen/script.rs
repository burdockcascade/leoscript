use std::collections::HashMap;
use std::env::current_dir;
use std::fs;
use std::path::Path;

use crate::compiler::codegen::CodeGenerationResult;
use crate::compiler::codegen::function::Function;
use crate::compiler::codegen::syntax::{Syntax, TokenPosition};
use crate::compiler::error::{CompilerError, CompilerErrorType};
use crate::compiler::parser::Parser;
use crate::compiler::warning::{CompilerWarning, CompilerWarningType};
use crate::runtime::ir::instruction::Instruction;
use crate::runtime::ir::variant::Variant;

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

const TYPE_FIELD: &str = "_type";

pub fn generate_script(source: &str, offset: usize) -> Result<CodeGenerationResult, CompilerError> {
    let mut script = CodeGenerationResult::default();

    let p = Parser::parse(source);

    // get tokens
    let parser_result = match p {
        Ok(r) => {
            if r.syntax_tree.len() == 0 {
                return Err(CompilerError {
                    error: CompilerErrorType::NoTokensGenerated,
                    position: Default::default(),
                });
            } else {
                r
            }
        },
        Err(e) => {
            return Err(CompilerError {
                error: CompilerErrorType::ParseError(e.error),
                position: e.position.clone(),
            })
        }
    };

    //println!("parser_result.tokens: {:#?}", parser_result.tokens);

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
    for token in parser_result.syntax_tree.clone() {
        match token {
            Syntax::Import { position, source, .. } => {

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
                    if let Syntax::Identifier { name, .. } = identifier {
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
    for token in parser_result.syntax_tree.clone() {
        let local_offset = script.instructions.len() + offset;

        match token {
            Syntax::Function { position, function_name, parameters: input, body, .. } => {
                let func = Function::new(position, function_name.to_string(), input, body)?;
                script.globals.insert(function_name.to_string(), Variant::FunctionPointer(local_offset));
                script.instructions.append(&mut func.instructions.clone());
            }
            //Syntax::Module { .. } => generate_module(script, token, local_offset)?,
            Syntax::Class { position, class_name, constructor, attributes, methods } => {
                let class_name_as_string = class_name.to_string();
                let class_struct = generate_class(position, class_name, attributes, constructor, methods, local_offset)?;

                script.globals.insert(class_name_as_string, Variant::Class(class_struct.structure));
                script.instructions.append(&mut class_struct.instructions.clone());
            }
            Syntax::Enum { position, name, items } => {
                let enum_def = generate_enum(position, name.clone(), items)?;
                script.globals.insert(name.to_string(), enum_def);
            }
            _ => {}
        }
    }

    script.compiler_time += start_compiler_timer.elapsed();

    Ok(script)
}

pub fn generate_class(position: TokenPosition, name: Box<Syntax>, attributes: Vec<Syntax>, constructor: Option<Box<Syntax>>, methods: Vec<Syntax>, ip_offset: usize) -> Result<CodeStructure, CompilerError> {

    let mut c = CodeStructure::default();

    c.structure.insert(String::from(TYPE_FIELD), Variant::Type(name.to_string()));

    // Attributes

    for attr in attributes.clone() {
        match attr {
            Syntax::Attribute { name, .. } => {
                c.structure.insert(name.to_string(), Variant::Null);
            },
            _ => {}
        }
    }

    // Constructor

    let f = match constructor {
        Some(c) => {
            let Syntax::Constructor { position, input, body } = *c else { todo!() };
            generate_constructor(position, input, body, attributes)?
        },
        _ => generate_constructor(position, vec![], vec![], attributes)?
    };

    c.structure.insert(String::from(CONSTRUCTOR_NAME), Variant::FunctionPointer(c.instructions.len() + ip_offset));
    c.instructions.append(&mut f.instructions.clone());

    // Methods

    for method in methods.clone() {
        match method {
            Syntax::Function { position, function_name, is_static, mut parameters, body, .. } => {
                // add self to the input if not static
                if !is_static {
                    parameters.insert(0, Syntax::Variable {
                        position: TokenPosition::default(),
                        name: Box::new(Syntax::Identifier {
                            position: TokenPosition::default(),
                            name: String::from(SELF_CONSTANT),
                        }),
                        as_type: None,
                        value: None,
                    });
                }

                let func = Function::new(position, function_name.to_string(), parameters, body)?;
                c.structure.insert(function_name.to_string(), Variant::FunctionPointer(c.instructions.len() + ip_offset));
                c.instructions.append(&mut func.instructions.clone());
            }
            _ => unreachable!("Class methods should be functions")
        }
    }

    Ok(c)
}

fn generate_constructor(position: TokenPosition, mut input: Vec<Syntax>, mut body: Vec<Syntax>, attributes: Vec<Syntax>) -> Result<Function, CompilerError> {

    // first parameter is always self
    input.insert(0, Syntax::Variable {
        position: TokenPosition::default(),
        name: Box::new(Syntax::Identifier {
            position: TokenPosition::default(),
            name: String::from(SELF_CONSTANT),
        }),
        as_type: None,
        value: None,
    });

    // constructor returns self
    body.push(Syntax::Return {
        position: TokenPosition::default(),
        expr: Some(Box::new(Syntax::Identifier {
            position: TokenPosition::default(),
            name: String::from(SELF_CONSTANT),
        })),
    });

    // add properties to class
    for attr in attributes.clone() {
        match attr {
            Syntax::Attribute { name, value, .. } => {
                body.insert(0, Syntax::Assign {
                    position: TokenPosition::default(),
                    target: Box::new(Syntax::MemberAccess {
                        position: TokenPosition::default(),
                        target: Box::new(Syntax::Identifier {
                            position: TokenPosition::default(),
                            name: String::from(SELF_CONSTANT),
                        }),
                        index: Box::new(Syntax::Identifier {
                            position: TokenPosition::default(),
                            name: name.to_string(),
                        }),
                    }),
                    value: value.unwrap_or_else(|| Box::from(Syntax::Null)),
                });
            }
            _ => unreachable!("Class member should be attribute")
        }
    }

    Function::new(position, String::from(CONSTRUCTOR_NAME), input, body)
}

pub fn generate_module(script: &mut CodeGenerationResult, syntax: Syntax, ip_offset: usize) -> Result<CodeStructure, CompilerError> {
    let mut fgroup = CodeStructure::default();

    let Syntax::Module { position: _, module_name, constants: _, functions, classes, enums, modules, imports: _ } = syntax else {
        panic!("generate_module called with non-module syntax")
    };

    // set module type
    fgroup.structure.insert(String::from(TYPE_FIELD), Variant::Type(module_name.to_string()));

    // functions
    for function in functions {
        match function {
            Syntax::Function { position, function_name, parameters: mut input, body, .. } => {
                input.insert(0, Syntax::Variable {
                    position: TokenPosition::default(),
                    name: Box::new(
                        Syntax::Identifier {
                            position: TokenPosition::default(),
                            name: String::from(SELF_CONSTANT),
                        }
                    ),
                    as_type: None,
                    value: None,
                });

                let func = Function::new(position, function_name.to_string(), input, body)?;
                fgroup.structure.insert(function_name.to_string(), Variant::FunctionPointer(fgroup.instructions.len() + ip_offset));
                fgroup.instructions.append(&mut func.instructions.clone());
            }
            _ => {}
        }
    }

    // classes
    for class in classes {
        match class {
            Syntax::Class { position, class_name, attributes, constructor, methods} => {
                let class_name_as_string = class_name.to_string();
                let class_struct = generate_class(position, class_name, attributes, constructor, methods, fgroup.instructions.len() + ip_offset)?;

                fgroup.structure.insert(class_name_as_string, Variant::Class(class_struct.structure));
                fgroup.instructions.append(&mut class_struct.instructions.clone());
            }
            _ => {}
        }
    }

    // enums
    for enum_item in enums {
        match enum_item {
            Syntax::Enum { position, name, items } => {
                let enum_def = generate_enum(position, name.clone(), items)?;
                fgroup.structure.insert(name.to_string(), enum_def);
            }
            _ => {}
        }
    }

    // modules
    for module in modules {
        match module {
            Syntax::Module { .. } => {
                let module_name_as_string = module_name.to_string();
                let module_struct = generate_module(script, module, fgroup.instructions.len() + ip_offset)?;

                fgroup.structure.insert(module_name_as_string, Variant::Module(module_struct.structure));
                fgroup.instructions.append(&mut module_struct.instructions.clone());
            }
            _ => {}
        }
    }

    Ok(fgroup)
}

pub fn generate_enum(_position: TokenPosition, _name: Box<Syntax>, items: Vec<Syntax>) -> Result<Variant, CompilerError> {
    let mut enum_def = HashMap::default();

    let mut index = 0;

    for item in items {
        enum_def.insert(item.to_string(), index);
        index += 1;
    }

    Ok(Variant::Enum(enum_def))
}