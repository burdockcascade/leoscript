use leoscript_runtime::ir::variant::Variant;
use crate::compiler::class::compile_class;
use crate::compiler::CompilerError;
use crate::compiler::function::Function;
use crate::compiler::r#enum::compile_enum;
use crate::compiler::script::{CodeStructure, SELF_CONSTANT};
use crate::parser::token::{Token, TokenPosition};

const MODULE_TYPE_FIELD: &str = "_type";

pub fn compile_module(_position: TokenPosition, name: Box<Token>, body: Vec<Token>, ip_offset: usize) -> Result<CodeStructure, CompilerError> {
    let mut fgroup = CodeStructure::default();

    // set module type
    fgroup.structure.insert(String::from(MODULE_TYPE_FIELD), Variant::Type(name.to_string()));

    for item in body.clone() {
        match item {
            Token::Constant { name, .. } => {
                fgroup.structure.insert(name.to_string(), Variant::Null); // fixme
            }
            Token::Function { position, function_name, mut input, body, .. } => {
                input.insert(0, Token::Variable {
                    position: TokenPosition::default(),
                    name: String::from(SELF_CONSTANT),
                    as_type: None,
                    value: None,
                });

                let func = Function::new(position, function_name.to_string(), input, body)?;
                fgroup.structure.insert(function_name.to_string(), Variant::FunctionPointer(fgroup.instructions.len() + ip_offset));
                fgroup.instructions.append(&mut func.instructions.clone());
            }
            Token::Class { position, class_name, body, .. } => {
                let class_name_as_string = class_name.to_string();
                let class_struct = compile_class(position, class_name, body, fgroup.instructions.len() + ip_offset)?;

                fgroup.structure.insert(class_name_as_string, Variant::Class(class_struct.structure));
                fgroup.instructions.append(&mut class_struct.instructions.clone());
            }
            Token::Module { position, module_name, body, .. } => {
                let module_name_as_string = module_name.to_string();
                let mod_struct = compile_module(position, module_name, body, fgroup.instructions.len() + ip_offset)?;

                fgroup.structure.insert(module_name_as_string, Variant::Module(mod_struct.structure));
                fgroup.instructions.append(&mut mod_struct.instructions.clone());
            }
            Token::Enum { position, name, items } => {
                let enum_def = compile_enum(position, name.clone(), items)?;
                fgroup.structure.insert(name.to_string(), enum_def);
            }
            _ => {}
        }
    };

    Ok(fgroup)
}