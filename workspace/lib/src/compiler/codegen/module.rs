use crate::compiler::codegen::class::generate_class;
use crate::compiler::codegen::function::Function;
use crate::compiler::codegen::r#enum::generate_enum;
use crate::compiler::codegen::script::{CodeStructure, SELF_CONSTANT};
use crate::compiler::codegen::syntax::{Syntax, TokenPosition};
use crate::compiler::error::CompilerError;
use crate::runtime::ir::variant::Variant;

const MODULE_TYPE_FIELD: &str = "_type";

pub fn generate_module(_position: TokenPosition, name: Box<Syntax>, body: Vec<Syntax>, ip_offset: usize) -> Result<CodeStructure, CompilerError> {
    let mut fgroup = CodeStructure::default();

    // set module type
    fgroup.structure.insert(String::from(MODULE_TYPE_FIELD), Variant::Type(name.to_string()));

    for item in body.clone() {
        match item {
            Syntax::Constant { name, .. } => {
                fgroup.structure.insert(name.to_string(), Variant::Null); // fixme
            }
            Syntax::Function { position, function_name, mut input, body, .. } => {
                input.insert(0, Syntax::Variable {
                    position: TokenPosition::default(),
                    name: String::from(SELF_CONSTANT),
                    as_type: None,
                    value: None,
                });

                let func = Function::new(position, function_name.to_string(), input, body)?;
                fgroup.structure.insert(function_name.to_string(), Variant::FunctionPointer(fgroup.instructions.len() + ip_offset));
                fgroup.instructions.append(&mut func.instructions.clone());
            }
            Syntax::Class { position, class_name, body, .. } => {
                let class_name_as_string = class_name.to_string();
                let class_struct = generate_class(position, class_name, body, fgroup.instructions.len() + ip_offset)?;

                fgroup.structure.insert(class_name_as_string, Variant::Class(class_struct.structure));
                fgroup.instructions.append(&mut class_struct.instructions.clone());
            }
            Syntax::Module { position, module_name, body, .. } => {
                let module_name_as_string = module_name.to_string();
                let mod_struct = generate_module(position, module_name, body, fgroup.instructions.len() + ip_offset)?;

                fgroup.structure.insert(module_name_as_string, Variant::Module(mod_struct.structure));
                fgroup.instructions.append(&mut mod_struct.instructions.clone());
            }
            Syntax::Enum { position, name, items } => {
                let enum_def = generate_enum(position, name.clone(), items)?;
                fgroup.structure.insert(name.to_string(), enum_def);
            }
            _ => {}
        }
    };

    Ok(fgroup)
}