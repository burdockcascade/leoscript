use nom_locate::LocatedSpan;

pub mod token;
pub mod parser;

type Span<'a> = LocatedSpan<&'a str>;
