use crate::combinator::{ParserError, ParserInput, ParserPosition};

macro_rules! position_counter {
    ($pp:expr, $result:ident) => {
        {
            let mut line_counter = 0;
            let mut column_counter = 0;

            for c in $result.chars() {
                if c == '\n' {
                    line_counter += 1;
                    column_counter = 1;
                } else {
                    column_counter += 1;
                }
            };

            ParserPosition {
                line: $pp.line + line_counter,
                column: if line_counter > 0 { column_counter } else { $pp.column + column_counter }
            }
        }
    };
}

pub fn take<'a>(length: usize) -> impl Fn(&ParserInput<'a>) -> Result<(ParserInput<'a>, &'a str), ParserError> {
    move |input: &ParserInput<'a>| {
        if input.source.len() < length {
            return Err(ParserError::EndOfData {
                position: input.position.clone()
            });
        }
        let (result, source) = input.source.split_at(length);
        Ok((ParserInput {
            source,
            position: position_counter!(input.position, result)
        }, result))
    }
}

pub fn take_while<'a, P>(predicate: P) -> impl Fn(&ParserInput<'a>) -> Result<(ParserInput<'a>, &'a str), ParserError>
    where P: Fn(char) -> bool
{
    move |input: &ParserInput<'a>| {
        let mut chars = input.source.chars();
        let mut length = 0;
        while let Some(c) = chars.next() {
            if !predicate(c) {
                break;
            }
            length += 1;
        }
        let (result, source) = input.source.split_at(length);
        Ok((ParserInput {
            source,
            position: position_counter!(input.position, result)
        }, result))
    }
}

pub fn take_until<'a, P>(predicate: P) -> impl Fn(&ParserInput<'a>) -> Result<(ParserInput<'a>, &'a str), ParserError>
    where P: Fn(char) -> bool
{
    move |input: &ParserInput<'a>| {
        let mut chars = input.source.chars();
        let mut length = 0;
        while let Some(c) = chars.next() {
            if predicate(c) {
                break;
            }
            length += 1;
        }
        let (result, source) = input.source.split_at(length);
        Ok((ParserInput {
            source,
            position: position_counter!(input.position, result)
        }, result))
    }
}

#[cfg(test)]
mod test {

    use super::*;
    
    #[test]
    fn test_take_ok() {
        let input = ParserInput::new("hello world");
        let output: Result<(ParserInput, &str), ParserError> = take(5)(&input);
        assert_eq!(output, Ok((ParserInput {
            source: " world",
            position: ParserPosition {
                line: 1,
                column: 6,
            }
        }, "hello")));
    }

    #[test]
    fn test_take_multiline_ok() {
        let input = ParserInput::new("hello\nworld");
        let output: Result<(ParserInput, &str), ParserError> = take(6)(&input);
        assert_eq!(output, Ok((ParserInput {
            source: "world",
            position: ParserPosition {
                line: 2,
                column: 1,
            }
        }, "hello\n")));
    }

    #[test]
    fn test_take_fail() {
        let input = ParserInput::new("hello");
        let output: Result<(ParserInput, &str), ParserError> = take(6)(&input);
        assert_eq!(output, Err(ParserError::EndOfData {
            position: ParserPosition {
                line: 1,
                column: 1,
            }
        }));
    }

    #[test]
    fn test_take_while_ok() {
        let input = ParserInput::new("hello world");
        let output: Result<(ParserInput, &str), ParserError> = take_while(|c| c.is_alphabetic())(&input);
        assert_eq!(output, Ok((ParserInput {
            source: " world",
            position: ParserPosition {
                line: 1,
                column: 6,
            }
        }, "hello")));
    }

    #[test]
    fn test_take_while_fail() {
        let input = ParserInput::new("123 hello world");
        let output: Result<(ParserInput, &str), ParserError> = take_while(|c| c.is_alphabetic())(&input);
        assert_eq!(output, Ok((ParserInput {
            source: "123 hello world",
            position: ParserPosition {
                line: 1,
                column: 1,
            }
        }, "")));
    }
    
}