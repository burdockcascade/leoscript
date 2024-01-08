use std::collections::HashMap;
use crate::compiler::codegen::CodeGenerationResult;
use crate::compiler::codegen::function::Function;
use crate::compiler::codegen::syntax::{Syntax, TokenPosition};
use crate::compiler::error::CodegenError;
use crate::compiler::parser::Parser;
use crate::runtime::ir::instruction::Instruction;
use crate::runtime::ir::variant::{CLASS_CONSTRUCTOR_NAME, Variant};

const FILE_EXTENSION: &str = ".leo";
pub const SELF_CONSTANT: &str = "self";
const TYPE_FIELD: &str = "_type";

#[derive(Debug)]
pub struct Script {
    pub structure: HashMap<String, Variant>,
    pub instructions: Vec<Instruction>,
}

impl Script {

    pub fn new() -> Self {
        Script {
            structure: HashMap::new(),
            instructions: Vec::new(),
        }
    }

    pub fn generate_script(&mut self, source: Vec<Syntax>) -> Result<(), CodegenError> {
        for token in source.clone() {
            match token {
                Syntax::Function { position, function_name, parameters: input, body, .. } => {
                    let v = self.generate_function(position, function_name.to_string(), input, body)?;
                    self.structure.insert(function_name.to_string(), v);
                }
                Syntax::Module { position, module_name, body } => {
                    let v= self.generate_module(position, module_name.to_string(), body)?;
                    self.structure.insert(module_name.to_string(), v);
                },
                Syntax::Class { position, class_name, constructor, attributes, methods } => {
                    let v= self.generate_class(position, class_name.to_string(), attributes, constructor, methods)?;
                    self.structure.insert(class_name.to_string(), v);
                }
                Syntax::Enum { position, name, items } => {
                    let v = self.generate_enum(position, name.to_string(), items)?;
                    self.structure.insert(name.to_string(), v);
                }
                _ => {}
            }
        }

        Ok(())
    }

    fn generate_function(&mut self, position: TokenPosition, name: String, input: Vec<Syntax>, body: Vec<Syntax>) -> Result<Variant, CodegenError> {
        let func = Function::new(position, name.clone(), input, body)?;
        let fp = Variant::FunctionPointer(self.instructions.len());
        self.instructions.append(&mut func.instructions.clone());
        Ok(fp)
    }

    fn generate_method(&mut self, position: TokenPosition, name: String, mut input: Vec<Syntax>, body: Vec<Syntax>) -> Result<Variant, CodegenError> {
        input.insert(0, Syntax::Variable {
            position: TokenPosition::default(),
            name: Box::new(Syntax::Identifier {
                position: TokenPosition::default(),
                name: String::from(SELF_CONSTANT),
            }),
            value: None,
        });

        self.generate_function(position, name, input, body)
    }

    fn generate_class(&mut self, position: TokenPosition, name: String, attributes: Vec<Syntax>, constructor: Option<Box<Syntax>>, methods: Vec<Syntax>) -> Result<Variant, CodegenError> {

        let mut structure = HashMap::default();
        structure.insert(String::from(TYPE_FIELD), Variant::Type(name));

        // Attributes

        for attr in attributes.clone() {
            match attr {
                Syntax::Attribute { name, .. } => {
                    structure.insert(name.to_string(), Variant::Null);
                },
                _ => {}
            }
        }

        // Constructor
        let v = match constructor {
            Some(c) => {
                let Syntax::Constructor { position, input, body } = *c else { todo!() };
                self.generate_constructor(position, input, body, attributes)?
            },
            _ => self.generate_constructor(position, vec![], vec![], attributes)?
        };

        structure.insert(String::from(CLASS_CONSTRUCTOR_NAME), v);

        // Methods
        for method in methods.clone() {
            match method {
                Syntax::Function { position, function_name, is_static, parameters, body, .. } => {
                    // add self to the input if not static
                    let fp = if is_static {
                        self.generate_function(position, function_name.to_string(), parameters, body)?
                    } else {
                        self.generate_method(position, function_name.to_string(), parameters, body)?
                    };

                    structure.insert(function_name.to_string(), fp);
                }
                _ => unreachable!("Class methods should be functions")
            }
        }

        Ok(Variant::Class(structure))
    }

    fn generate_constructor(&mut self, position: TokenPosition, mut input: Vec<Syntax>, mut body: Vec<Syntax>, attributes: Vec<Syntax>) -> Result<Variant, CodegenError> {

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

        self.generate_method(position, String::from(CLASS_CONSTRUCTOR_NAME), input, body)
    }

    pub fn generate_module(&mut self, position: TokenPosition, name: String, body: Vec<Syntax>) -> Result<Variant, CodegenError> {

        let mut structure = HashMap::default();

        // set module type
        structure.insert(String::from(TYPE_FIELD), Variant::Type(name));

        // functions
        for member in body {
            match member {
                Syntax::Function { position, function_name, parameters: input, body, .. } => {
                    let v = self.generate_function(position, function_name.to_string(), input, body)?;
                    structure.insert(function_name.to_string(), v);
                }
                Syntax::Class { position, class_name, attributes, constructor, methods} => {
                    let v = self.generate_class(position, class_name.to_string(), attributes, constructor, methods)?;
                    structure.insert(class_name.to_string(), v);
                }
                Syntax::Enum { position, name, items } => {
                    let v = self.generate_enum(position, name.to_string(), items)?;
                    structure.insert(name.to_string(), v);
                }
                Syntax::Module { position, module_name, body } => {
                    let v = self.generate_module(position, module_name.to_string(), body)?;
                    structure.insert(module_name.to_string(), v);
                }
                _ => unimplemented!("generate_module function")
            }
        }

        Ok(Variant::Module(structure))
    }

    pub fn generate_enum(&mut self, _position: TokenPosition, name: String, items: Vec<Syntax>) -> Result<Variant, CodegenError> {
        let mut enum_def = HashMap::default();

        let mut index = 0;

        for item in items {
            enum_def.insert(item.to_string(), index);
            index += 1;
        }

        Ok(Variant::Enum(enum_def))
    }

}
