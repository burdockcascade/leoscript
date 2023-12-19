use nom_locate::LocatedSpan;

pub mod token;
pub mod parser;
mod comments;

type Span<'a> = LocatedSpan<&'a str>;
