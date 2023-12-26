use crate::runtime::ir::variant::ValueType;

#[derive(Clone)]
pub struct Variable {
    pub slot_index: usize,
    pub name: String,
    pub as_type: ValueType,
}