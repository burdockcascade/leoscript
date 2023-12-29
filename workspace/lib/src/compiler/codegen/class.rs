use crate::compiler::codegen::function::Function;
use crate::compiler::codegen::script::{CodeStructure, CONSTRUCTOR_NAME, SELF_CONSTANT};
use crate::compiler::codegen::syntax::{Syntax, TokenPosition};
use crate::compiler::error::CompilerError;
use crate::runtime::ir::variant::Variant;

const CLASS_TYPE_FIELD: &str = "_type";

pub fn generate_class(position: TokenPosition, name: Box<Syntax>, body: Vec<Syntax>, ip_offset: usize) -> Result<CodeStructure, CompilerError> {

    let mut c = CodeStructure::default();

    c.structure.insert(String::from(CLASS_TYPE_FIELD), Variant::Type(name.to_string()));

    let mut properties: Vec<Syntax> = Vec::default();

    // collect class properties
    for item in body.clone() {
        match item.clone() {
            Syntax::Attribute { name, .. } => {
                properties.push(item);
                c.structure.insert(name.to_string(), Variant::Null);
            }
            _ => {}
        }
    }

    for item in body.clone() {
        match item {
            Syntax::Constructor { position, input, body } => {
                let f = generate_constructor(position, input, body, properties.clone())?;
                c.structure.insert(String::from(CONSTRUCTOR_NAME), Variant::FunctionPointer(c.instructions.len() + ip_offset));
                c.instructions.append(&mut f.instructions.clone());
            }
            Syntax::Function { position, function_name, is_static, mut input, body, .. } => {

                // add self to the input if not static
                if !is_static {
                    input.insert(0, Syntax::Variable {
                        position: TokenPosition::default(),
                        name: String::from(SELF_CONSTANT),
                        as_type: None,
                        value: None,
                    });
                }

                let func = Function::new(position, function_name.to_string(), input, body)?;
                c.structure.insert(function_name.to_string(), Variant::FunctionPointer(c.instructions.len() + ip_offset));
                c.instructions.append(&mut func.instructions.clone());
            }
            Syntax::Constant { .. } => unimplemented!("Constants not implemented yet"),
            _ => {}
        }
    }

    // add default constructor if not defined
    if !c.structure.contains_key(CONSTRUCTOR_NAME) {
        let f = generate_constructor(position, vec![], vec![], properties)?;
        c.structure.insert(String::from(CONSTRUCTOR_NAME), Variant::FunctionPointer(c.instructions.len() + ip_offset));
        c.instructions.append(&mut f.instructions.clone());
    }

    Ok(c)
}

fn generate_constructor(position: TokenPosition, mut input: Vec<Syntax>, mut body: Vec<Syntax>, properties: Vec<Syntax>) -> Result<Function, CompilerError> {

    // first parameter is always self
    input.insert(0, Syntax::Variable {
        position: TokenPosition::default(),
        name: String::from(SELF_CONSTANT),
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
    for prop in properties.clone() {
        match prop {
            Syntax::Attribute { name, value, .. } => {

                // add property to class
                body.insert(0, Syntax::Assign {
                    position: TokenPosition::default(),
                    ident: Box::new(Syntax::DotChain {
                        position: TokenPosition::default(),
                        start: Box::new(Syntax::Identifier {
                            position: TokenPosition::default(),
                            name: String::from(SELF_CONSTANT),
                        }),
                        chain: vec![Syntax::Identifier {
                            position: TokenPosition::default(),
                            name: name.to_string(),
                        }],
                    }),
                    value: value.unwrap_or_else(|| Box::from(Syntax::Null)),
                });
            }
            _ => {}
        }
    }

    Function::new(position, String::from(CONSTRUCTOR_NAME), input, body)
}