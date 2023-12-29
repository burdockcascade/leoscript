use crate::compiler::codegen::syntax::{Syntax, TokenPosition};
use crate::compiler::error::{ParserError, ParserErrorType};
use crate::compiler::parser::ParserResult;
use crate::compiler::tokenizer::{get_tokenizer, Token};
use crate::compiler::tokenizer::lexer::{Cursor, MatchedToken, Tokenizer};
use crate::next_token;

pub fn parse_script(source: &str) -> Result<ParserResult, ParserError> {

    let parser_timer = std::time::Instant::now();

    let lexer = &mut get_tokenizer(source);
    let mut tokens = vec![];

    while !lexer.is_eof()  {

        let matched = next_token!(lexer);

        match matched.token {
            Token::Function => tokens.push(parse_function(matched.cursor, lexer)?),
            _ => {
                println!("peek: {:?}", matched.token);
            }
        }
    }

    Ok(ParserResult {
        tokens,
        parser_time: parser_timer.elapsed(),
    })
}

fn parse_function(cursor: Cursor, lexer: &mut Tokenizer<Token>) -> Result<Syntax, ParserError> {

    // function name
    let function_name = parse_identifier(lexer)?;

    // functiona args
    let args = next_token!(lexer);
    match args.token {
        Token::NoArgs => {},
        _ => todo!("args")
    }

    // function end
    let _ = next_token!(lexer);

    Ok(Syntax::Function {
        position: TokenPosition {
            line: cursor.line,
            column: cursor.column,
        },
        function_name: Box::new(function_name),
        is_static: false,
        scope: None,
        return_type: None,
        input: vec![],
        body: vec![],
    })
}

fn parse_identifier(lexer: &mut Tokenizer<Token>) -> Result<Syntax, ParserError> {

    // function name
    let matched = next_token!(lexer);

    // return identifier
    match matched.token {
        Token::Identifier => Ok(Syntax::Identifier {
            position: TokenPosition {
                line: matched.cursor.line,
                column: matched.cursor.column,
            },
            name: matched.text.to_string(),
        }),
        _ => {
            return Err(ParserError {
                error: ParserErrorType::InvalidIdentifier(matched.text),
                position: TokenPosition {
                    line: matched.cursor.line,
                    column: matched.cursor.column,
                },
            })
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_script() {

        let source = r#"
            function main()
            end
        "#;

        let Ok(r) = parse_script(source) else {
            assert!(false, "bad parse");
            return;
        };

        assert_eq!(r.tokens, vec![
            Syntax::Function {
                position: TokenPosition { line: 2, column: 13 },
                function_name: Box::new(Syntax::Identifier {
                    position: TokenPosition { line: 2, column: 22 },
                    name: String::from("main"),
                }),
                is_static: false,
                scope: None,
                return_type: None,
                input: vec![],
                body: vec![],
            }
        ])

    }

}