use crate::combinator::{ParserError, ParserInput};

pub fn delimited<'a, P1, P2, P3, R>(open: P1, parser: P2, close: P3) -> impl Fn(&ParserInput<'a>) -> Result<(ParserInput<'a>, R), ParserError>
    where P1: Fn(&ParserInput<'a>) -> Result<(ParserInput<'a>, &'a str), ParserError>,
          P2: Fn(&ParserInput<'a>) -> Result<(ParserInput<'a>, R), ParserError>,
          P3: Fn(&ParserInput<'a>) -> Result<(ParserInput<'a>, &'a str), ParserError>,
{
    move |input: &ParserInput<'a>| {
        let (input, _) = open(input)?;
        let (input, result) = parser(&input)?;
        let (input, _) = close(&input)?;
        Ok((input, result))
    }
}

pub fn preceded<'a, P1, P2, R>(prefix: P1, parser: P2) -> impl Fn(&ParserInput<'a>) -> Result<(ParserInput<'a>, R), ParserError>
    where P1: Fn(&ParserInput<'a>) -> Result<(ParserInput<'a>, &'a str), ParserError>,
          P2: Fn(&ParserInput<'a>) -> Result<(ParserInput<'a>, R), ParserError>,
{
    move |input: &ParserInput<'a>| {
        let (input, _) = prefix(input)?;
        parser(&input)
    }
}

pub fn terminated<'a, P1, P2, R>(parser: P1, suffix: P2) -> impl Fn(&ParserInput<'a>) -> Result<(ParserInput<'a>, R), ParserError>
    where P1: Fn(&ParserInput<'a>) -> Result<(ParserInput<'a>, R), ParserError>,
          P2: Fn(&ParserInput<'a>) -> Result<(ParserInput<'a>, &'a str), ParserError>,
{
    move |input: &ParserInput<'a>| {
        let (input, result) = parser(input)?;
        let (input, _) = suffix(&input)?;
        Ok((input, result))
    }
}

mod test {
    use crate::combinator::matcher::{alpha, numeric, token};
    use crate::combinator::ParserPosition;
    use super::*;

    #[test]
    fn test_delimited_ok() {
        let input = ParserInput::new("(hello)");
        let output: Result<(ParserInput, &str), ParserError> = delimited(token("("), token("hello"), token(")"))(&input);
        assert_eq!(output, Ok((ParserInput {
            source: "",
            position: ParserPosition {
                line: 1,
                column: 8,
            }
        }, "hello")));
    }

    #[test]
    fn test_delimited_fail() {
        let input = ParserInput::new("hello");
        let output: Result<(ParserInput, &str), ParserError> = delimited(token("("), token("goodbye"), token(")"))(&input);
        assert_eq!(output, Err(ParserError::ExpectedToken {
            expected: '(',
            found: 'h',
            position: ParserPosition {
                line: 1,
                column: 1,
            }
        }));
    }

    #[test]
    fn test_preceded_ok() {
        let input = ParserInput::new("hello world");
        let output: Result<(ParserInput, &str), ParserError> = preceded(token("hello"), token(" world"))(&input);
        assert_eq!(output, Ok((ParserInput {
            source: "",
            position: ParserPosition {
                line: 1,
                column: 12,
            }
        }, " world")));
    }

    #[test]
    fn test_preceded_fail() {
        let input = ParserInput::new("hello world");
        let output: Result<(ParserInput, &str), ParserError> = preceded(token("goodbye"), token(" world"))(&input);
        assert_eq!(output, Err(ParserError::ExpectedToken {
            expected: 'g',
            found: 'h',
            position: ParserPosition {
                line: 1,
                column: 1,
            }
        }));
    }

    #[test]
    fn test_terminated_uk() {
        let input = ParserInput::new("hello world");
        let output: Result<(ParserInput, &str), ParserError> = terminated(token("hello"), token(" world"))(&input);
        assert_eq!(output, Ok((ParserInput {
            source: "",
            position: ParserPosition {
                line: 1,
                column: 12,
            }
        }, "hello")));
    }

    #[test]
    fn test_terminated_fail() {
        let input = ParserInput::new("hello world");
        let output: Result<(ParserInput, &str), ParserError> = terminated(token("hello "), token("goodbye"))(&input);
        assert_eq!(output, Err(ParserError::ExpectedToken {
            expected: 'w',
            found: 'g',
            position: ParserPosition {
                line: 1,
                column: 7,
            }
        }));
    }
    
}