
#[macro_export]
macro_rules! script_compile_warning {
    ($warning:expr, $position:expr) => {
        ScriptWarning::CompilerWarning {
            warning: $warning,
            line: $position.line,
            column: $position.column,
        }
    };
    ($warning:expr) => {
        ScriptWarning::CompilerWarning {
            warning: $warning,
            line: 0,
            column: 0,
        }
    };
}

#[derive(Debug, PartialEq)]
pub enum ScriptWarning {
    CompilerWarning {
        warning: CompilerWarning,
        line: usize,
        column: usize,
    }
}

#[derive(Debug, PartialEq)]
pub enum CompilerWarning {
    NothingToImport
}