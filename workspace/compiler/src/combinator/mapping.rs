use crate::combinator::{ParserError, ParserInput};

pub fn map<'a, F, A, B, E>(parser: F, f: impl Fn(A) -> B) -> impl Fn(&ParserInput<'a>) -> Result<(ParserInput<'a>, B), ParserError>
where
    F: Fn(&ParserInput<'a>) -> Result<(ParserInput<'a>, A), ParserError>,
    A: 'a,
    B: 'a,
    E: 'a
{
    move |input: &ParserInput<'a>| {
        let (taken, result) = parser(input)?;
        Ok((taken, f(result)))
    }
}

#[cfg(test)]
mod test {
    use crate::combinator::matcher::alpha;
    use crate::combinator::ParserPosition;
    use super::*;

    #[test]
    fn test_map_ok() {
        let input = ParserInput::new("hello");
        let output: Result<(ParserInput, &str), ParserError> = map(alpha(), |s| s.to_uppercase().as_str())(&input);
        assert_eq!(output, Ok((ParserInput {
            source: "",
            position: ParserPosition {
                line: 1,
                column: 6,
            }
        }, "HELLO")));
    }

}