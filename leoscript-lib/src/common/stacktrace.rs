#[derive(Clone, Debug, PartialEq)]
pub struct StackTrace {
    pub line: usize,
    pub file: String,
    pub function: String,
}