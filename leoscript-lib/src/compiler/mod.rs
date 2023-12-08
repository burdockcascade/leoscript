use nom_locate::LocatedSpan;

mod token;
mod parser;
pub mod script;
mod class;
mod function;
mod variable;
mod module;
mod r#enum;

type Span<'a> = LocatedSpan<&'a str>;