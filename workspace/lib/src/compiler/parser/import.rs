use std::fs;
use std::path::Path;

use crate::compiler::codegen::syntax::{Syntax, TokenPosition};
use crate::compiler::error::{ParserError, ParserErrorType};
use crate::compiler::parser::lexer::Token;
use crate::compiler::parser::Parser;
use crate::parser_error;

const FILE_EXTENSION: &str = ".leo";

impl Parser {
    pub(crate) fn parse_import(&mut self) -> Result<Vec<Syntax>, ParserError> {
        let import_keyword = self.next_token_or_error()?;

        let mut path = String::new();

        // read path
        while self.lexer.has_more_tokens() {
            let peeked = self.peek_next_token_or_error()?;

            match peeked.token {
                Token::Identifier => {
                    let identifier = self.next_token_or_error()?;
                    path += identifier.text.as_str();
                    if self.peek_next_token_match(Token::Dot) {
                        path += "/";
                        self.skip_next_token_or_error()?;
                    } else {
                        path += FILE_EXTENSION;
                    }
                }
                _ => break,
            }
        }

        // does file exist?
        if !Path::new(&path).exists() {
            return parser_error!(TokenPosition { line: import_keyword.cursor.line,column: import_keyword.cursor.column, }, ParserErrorType::InvalidImportPath(path));
        }

        // read imported file
        let contents = fs::read_to_string(path.clone()).unwrap();

        // parse imported file
        let p = Parser::parse(&contents)?;

        Ok(p.syntax_tree)
    }
}

#[cfg(test)]
mod test {
    use crate::compiler::parser::Parser;

    #[test]
    fn test_imports() {
        let main_source = r#"
            import tests.scripts.super

            function main()
                var d = Graphics::Dimension(10, 20)
                return d.area()
            end
        "#;

        let p = Parser::parse(main_source);

        assert!(p.is_ok());
    }
}