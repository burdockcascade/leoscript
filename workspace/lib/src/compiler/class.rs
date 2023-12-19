use std::collections::HashMap;

use log::trace;

use crate::common::error::ScriptError;
use crate::common::variant::Variant;
use crate::compiler::function::Function;
use crate::compiler::script::{CONSTRUCTOR_NAME, FunctionGroup, SELF_CONSTANT};
use crate::parser::token::{Token, TokenPosition};


pub fn compile_class(position: TokenPosition, name: Box<Token>, body: Vec<Token>, ip_offset: usize) -> Result<FunctionGroup, ScriptError> {
    trace!("Compiling class: {}", name);

    let mut c = FunctionGroup {
        structure: HashMap::default(),
        instructions: Vec::default(),
    };

    let mut properties: Vec<Token> = Vec::default();

    // collect class properties
    for item in body.clone() {
        match item.clone() {
            Token::Attribute { name, .. } => {
                properties.push(item);
                c.structure.insert(name.to_string(), Variant::Null);
            }
            _ => {}
        }
    }

    for item in body.clone() {
        match item {
            Token::Constructor { position, input, body } => {
                let f = compile_constructor(position, input, body, properties.clone())?;
                c.structure.insert(String::from(CONSTRUCTOR_NAME), Variant::FunctionPointer(c.instructions.len() + ip_offset));
                c.instructions.append(&mut f.instructions.clone());
            }
            Token::Function { position, function_name, is_static, mut input, body, .. } => {

                // add self to the input if not static
                if !is_static {
                    input.insert(0, Token::Variable {
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
            Token::Constant { .. } => unimplemented!("Constants not implemented yet"),
            _ => { }
        }
    }

    // add default constructor if not defined
    if !c.structure.contains_key(CONSTRUCTOR_NAME) {
        let f= compile_constructor(position, vec![], vec![], properties)?;
        c.structure.insert(String::from(CONSTRUCTOR_NAME), Variant::FunctionPointer(c.instructions.len() + ip_offset));
        c.instructions.append(&mut f.instructions.clone());
    }

    Ok(c)

}

fn compile_constructor(position: TokenPosition, mut input: Vec<Token>, mut body: Vec<Token>, properties: Vec<Token>) -> Result<Function, ScriptError> {

    // first parameter is always self
    input.insert(0, Token::Variable {
        position: TokenPosition::default(),
        name: String::from(SELF_CONSTANT),
        as_type: None,
        value: None,
    });

    // constructor returns self
    body.push(Token::Return {
        position: TokenPosition::default(),
        expr: Some(Box::new(Token::Identifier {
            position: TokenPosition::default(),
            name: String::from(SELF_CONSTANT),
        })),
    });

    // add properties to class
    for prop in properties.clone() {

        match prop {
            Token::Attribute { name, value, .. } => {

                // add property to class
                body.insert(0, Token::Assign {
                    position: TokenPosition::default(),
                    ident: Box::new(Token::DotChain {
                        position: TokenPosition::default(),
                        start: Box::new(Token::Identifier {
                            position: TokenPosition::default(),
                            name: String::from(SELF_CONSTANT),
                        }),
                        chain: vec![Token::Identifier {
                            position: TokenPosition::default(),
                            name: name.to_string(),
                        }],
                    }),
                    value: value.unwrap_or_else(|| Box::from(Token::Null)),
                });
            }
            _ => {}
        }

    }

    Function::new(position, String::from(CONSTRUCTOR_NAME), input, body)
}