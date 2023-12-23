use crate::combinator::mapping::map;
use crate::combinator::matcher::{alpha, whitespace};
use crate::combinator::{ParserError, ParserInput, ParserPosition};
use crate::combinator::sequence::preceded;
use crate::parser::token::{Token};

pub fn parse_identifier(input: ParserInput) -> Result<(ParserInput, Token), ParserError> {
    map(
        preceded(
            whitespace(),
            alpha(),
        ),
        |source| {
            Token::Identifier {
                position: ParserPosition {
                    line: input.position.line,
                    column: input.position.column,
                },
                name: source.to_string()
            }
        },
    )(&input)
}

mod test {
    use crate::combinator::ParserInput;
    use super::*;

    #[test]
    fn test_parse_identifier() {
        let input = ParserInput::new("hello");
        let result = parse_identifier(input);
        assert_eq!(result, Ok(("".to_string(), "hello".to_string())));
    }

}