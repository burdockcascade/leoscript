use crate::parser::token::TokenPosition;

#[derive(Debug, PartialEq)]
pub struct CompilerWarning {
    pub warning: CompilerWarningType,
    pub position: TokenPosition,
}

#[derive(Debug, PartialEq)]
pub enum CompilerWarningType {
    ImportFileEmpty(String),
}