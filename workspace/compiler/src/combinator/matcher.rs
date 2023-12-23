use crate::combinator::{ParserError, ParserInput};
use crate::combinator::taken::{take, take_while};

pub fn token<'a>(expected: &'a str) -> impl Fn(&ParserInput<'a>) -> Result<(ParserInput<'a>, &'a str), ParserError> {
    move |input: &ParserInput<'a>| {
        let (taken, result) = take(expected.len())(input)?;
        if result == expected {
            Ok((taken, result))
        } else {
            Err(ParserError::ExpectedToken {
                expected: expected.chars().next().unwrap(),
                found: result.chars().next().unwrap(),
                position: input.position.clone(),
            })
        }
    }
}

pub fn whitespace<'a>() -> impl Fn(&ParserInput<'a>) -> Result<(ParserInput<'a>, &'a str), ParserError> {
    move |input: &ParserInput<'a>| {
        let (taken, result) = take_while(|c| c.is_whitespace())(input)?;
        if result.len() > 0 {
            Ok((taken, result))
        } else {
            Err(ParserError::ExpectedWhitespace {
                found: input.source.chars().next().unwrap(),
                position: taken.position,
            })
        }
    }
}

pub fn alpha<'a>() -> impl Fn(&ParserInput<'a>) -> Result<(ParserInput<'a>, &'a str), ParserError> {
    move |input: &ParserInput<'a>| {
        let (taken, result) = take_while(|c| c.is_alphabetic())(input)?;
        if result.len() > 0 {
            Ok((taken, result))
        } else {
            Err(ParserError::ExpectedAlpha {
                found: input.source.chars().next().unwrap(),
                position: taken.position,
            })
        }
    }
}

pub fn numeric<'a>() -> impl Fn(&ParserInput<'a>) -> Result<(ParserInput<'a>, &'a str), ParserError> {
    move |input: &ParserInput<'a>| {
        let (taken, result) = take_while(|c| c.is_numeric())(input)?;
        if result.len() > 0 {
            Ok((taken, result))
        } else {
            Err(ParserError::ExpectedNumeric {
                found: input.source.chars().next().unwrap(),
                position: taken.position,
            })
        }
    }
}

pub fn alphanumeric<'a>() -> impl Fn(&ParserInput<'a>) -> Result<(ParserInput<'a>, &'a str), ParserError> {
    move |input: &ParserInput<'a>| {
        let (taken, result) = take_while(|c| c.is_alphanumeric())(input)?;
        if result.len() > 0 {
            Ok((taken, result))
        } else {
            Err(ParserError::ExpectedAlphanumeric {
                found: input.source.chars().next().unwrap(),
                position: taken.position,
            })
        }
    }
}

mod test {
    use crate::combinator::ParserPosition;
    use super::*;

    #[test]
    fn test_token_ok() {
        let input = ParserInput::new("hello world");


        let output: Result<(ParserInput, &str), ParserError> = token("hello")(&input);
        assert_eq!(output, Ok((ParserInput {
            source: " world",
            position: ParserPosition {
                line: 1,
                column: 6,
            }
        }, "hello")));
    }

    #[test]
    fn test_token_fail() {
        let input = ParserInput::new("hello world");
        let output: Result<(ParserInput, &str), ParserError> = token("goodbye")(&input);
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
    fn test_whitespace_ok() {
        let input = ParserInput::new("  hello world");
        let output: Result<(ParserInput, &str), ParserError> = whitespace()(&input);
        assert_eq!(output, Ok((ParserInput {
            source: "hello world",
            position: ParserPosition {
                line: 1,
                column: 3,
            }
        }, "  ")));
    }

    #[test]
    fn test_whitespace_multiline_ok() {
        let input = ParserInput::new("  \n   hello world");
        let output: Result<(ParserInput, &str), ParserError> = whitespace()(&input);
        assert_eq!(output, Ok((ParserInput {
            source: "hello world",
            position: ParserPosition {
                line: 2,
                column: 4,
            }
        }, "  \n   ")));
    }

    #[test]
    fn test_whitespace_fail() {
        let input = ParserInput::new("hello world");
        let output: Result<(ParserInput, &str), ParserError> = whitespace()(&input);
        assert_eq!(output, Err(ParserError::ExpectedWhitespace {
            found: 'h',
            position: ParserPosition {
                line: 1,
                column: 1,
            }
        }));
    }

    #[test]
    fn test_alpha_ok() {
        let input = ParserInput::new("hello world");
        let output: Result<(ParserInput, &str), ParserError> = alpha()(&input);
        assert_eq!(output, Ok((ParserInput {
            source: " world",
            position: ParserPosition {
                line: 1,
                column: 6,
            }
        }, "hello")));
    }

    #[test]
    fn test_alpha_fail() {
        let input = ParserInput::new("123 hello world");
        let output: Result<(ParserInput, &str), ParserError> = alpha()(&input);
        assert_eq!(output, Err(ParserError::ExpectedAlpha {
            found: '1',
            position: ParserPosition {
                line: 1,
                column: 1,
            }
        }));
    }

    #[test]
    fn test_numeric_ok() {
        let input = ParserInput::new("123 hello world");
        let output: Result<(ParserInput, &str), ParserError> = numeric()(&input);
        assert_eq!(output, Ok((ParserInput {
            source: " hello world",
            position: ParserPosition {
                line: 1,
                column: 4,
            }
        }, "123")));
    }

    #[test]
    fn test_numeric_fail() {
        let input = ParserInput::new("hello world");
        let output: Result<(ParserInput, &str), ParserError> = numeric()(&input);
        assert_eq!(output, Err(ParserError::ExpectedNumeric {
            found: 'h',
            position: ParserPosition {
                line: 1,
                column: 1,
            }
        }));
    }

    #[test]
    fn test_alphanumeric_ok() {
        let input = ParserInput::new("123 hello world");
        let output: Result<(ParserInput, &str), ParserError> = alphanumeric()(&input);
        assert_eq!(output, Ok((ParserInput {
            source: " hello world",
            position: ParserPosition {
                line: 1,
                column: 4,
            }
        }, "123")));
    }

    #[test]
    fn test_alphanumeric_fail() {
        let input = ParserInput::new("[hello world]");
        let output: Result<(ParserInput, &str), ParserError> = alphanumeric()(&input);
        assert_eq!(output, Err(ParserError::ExpectedAlphanumeric {
            found: '[',
            position: ParserPosition {
                line: 1,
                column: 1,
            }
        }));
    }
    
}