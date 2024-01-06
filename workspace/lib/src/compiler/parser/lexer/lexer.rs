use std::cmp::min;

use regex::{Error, Regex};

#[derive(Clone, Debug, PartialEq)]
pub enum LexerError {
    NoMatch,
    EndOfFile,
    InvalidRegularExpression(Error),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Cursor {
    pub position: usize,
    pub line: usize,
    pub column: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LexerOptions {
    pub ignore_whitespace: bool,
}

impl Default for LexerOptions {
    fn default() -> Self {
        Self {
            ignore_whitespace: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct MatchedToken<T> {
    pub token: T,
    pub text: String,
    pub cursor: Cursor,
}

macro_rules! track_position_by_char {
    ($cursor:expr, $c:expr) => {
        match $c {
            '\n' => {
                $cursor.line += 1;
                $cursor.column = 1;
            },
            _ => {
                $cursor.column += 1;
            },
        }
    };
}

#[macro_export]
macro_rules! match_token {
    ($condition:expr, $value:expr) => {
        Matcher::MatchToken {
            condition: String::from($condition),
            case_sensitive: false,
            value: $value,
        }
    };
    ($condition:expr, $value:expr, $case_sensitive:expr) => {
        Matcher::MatchToken {
            condition: String::from($condition),
            case_sensitive: $case_sensitive,
            value: $value,
        }
    };
}

#[macro_export]
macro_rules! ignore_token {
    ($condition:expr, $value:expr) => {
        Matcher::IgnoreToken {
            condition: String::from($condition),
            case_sensitive: false
        }
    };
    ($condition:expr, $value:expr, $case_sensitive:expr) => {
        Matcher::IgnoreToken {
            condition: String::from($condition),
            case_sensitive: $case_sensitive
        }
    };
}

#[macro_export]
macro_rules! match_regex {
    ($condition:expr, $value:expr) => {
        Matcher::MatchRegex {
            condition: String::from($condition),
            value: $value,
        }
    };
}

#[macro_export]
macro_rules! ignore_regex {
    ($condition:expr) => {
        Matcher::IgnoreRegex {
            condition: String::from($condition)
        }
    };
}

#[derive(Clone, Debug, PartialEq)]
pub enum Matcher<T> {
    MatchToken {
        condition: String,
        case_sensitive: bool,
        value: T,
    },
    MatchRegex {
        condition: String,
        value: T,
    },
    IgnoreToken {
        condition: String,
        case_sensitive: bool,
    },
    IgnoreRegex {
        condition: String,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Lexer<T> {
    matchers: Vec<Matcher<T>>,
    source: String,
    options: LexerOptions,
    cursor: Cursor,
}

impl<T> Default for Lexer<T> {
    fn default() -> Self {
        Self {
            matchers: Vec::new(),
            source: String::new(),
            options: LexerOptions::default(),
            cursor: Cursor {
                position: 0,
                line: 1,
                column: 1,
            },
        }
    }
}

impl<T: Clone + PartialEq> Lexer<T> {
    pub fn new(input: &str, matchers: Vec<Matcher<T>>, options: LexerOptions) -> Self {
        Self {
            matchers,
            source: String::from(input),
            options,
            cursor: Cursor {
                position: 0,
                line: 1,
                column: 1,
            },
        }
    }

    pub fn next(&mut self) -> Option<Result<MatchedToken<T>, LexerError>> {

        // check for EOF
        if self.is_eof() {
            return None;
        }

        let mut v: Option<Result<MatchedToken<T>, LexerError>> = None;

        // find match
        for matcher in self.matchers.iter() {
            match matcher {
                Matcher::MatchToken { condition, case_sensitive, value } => {
                    let cursor_position = self.cursor.position;

                    // get slice of condition length
                    let slice_end = min(cursor_position + condition.len(), self.source.len());
                    let slice = &self.source[cursor_position..slice_end];

                    // compare slice to condition
                    let token_match = if *case_sensitive {
                        slice == condition
                    } else {
                        slice.to_uppercase() == condition.to_uppercase()
                    };

                    // compare slice to condition
                    if token_match {
                        let text = &self.source[cursor_position..slice_end];

                        // set token
                        v = Some(Ok(MatchedToken {
                            token: value.clone(),
                            text: text.to_string(),
                            cursor: self.cursor.clone(),
                        }));

                        // increment cursor
                        self.cursor.position += condition.len();

                        for c in text.chars() {
                            track_position_by_char!(self.cursor, c);
                        }

                        // end search for matching token
                        break;
                    }
                }
                Matcher::IgnoreToken { condition, case_sensitive } => {
                    let cursor_position = self.cursor.position;

                    // get slice of condition length
                    let slice_end = min(cursor_position + condition.len(), self.source.len());
                    let slice = &self.source[cursor_position..slice_end];

                    // compare slice to condition
                    let token_match = if *case_sensitive {
                        slice == condition
                    } else {
                        slice.to_uppercase() == condition.to_uppercase()
                    };

                    // compare slice to condition
                    if token_match {

                        // increment cursor
                        self.cursor.position += condition.len();

                        for c in slice.chars() {
                            track_position_by_char!(self.cursor, c);
                        }

                        return self.next();
                    }
                }
                Matcher::MatchRegex { condition, value } => {
                    let cursor_position = self.cursor.position;

                    let re = match Regex::new(condition) {
                        Ok(re) => re,
                        Err(e) => {
                            return Some(Err(LexerError::InvalidRegularExpression(e)));
                        }
                    };

                    let caps = re.captures(&self.source[cursor_position..]);

                    if let Some(caps) = caps {

                        // get length of match
                        let caps_len = caps[0].len();

                        // get value
                        let text = &self.source[cursor_position..(cursor_position + caps_len)];

                        // set token
                        v = Some(Ok(MatchedToken {
                            token: value.clone(),
                            text: text.to_string(),
                            cursor: self.cursor.clone(),
                        }));

                        // increment cursor
                        self.cursor.position += caps_len;

                        for c in text.chars() {
                            track_position_by_char!(self.cursor, c);
                        }

                        // end search for matching token
                        break;
                    }
                }
                Matcher::IgnoreRegex { condition } => {
                    let cursor_position = self.cursor.position;

                    let re = match Regex::new(condition) {
                        Ok(re) => re,
                        Err(e) => {
                            return Some(Err(LexerError::InvalidRegularExpression(e)));
                        }
                    };

                    let caps = re.captures(&self.source[cursor_position..]);

                    if let Some(caps) = caps {

                        // get length of match
                        let caps_len = caps[0].len();

                        // get value
                        let text = &self.source[cursor_position..(cursor_position + caps_len)];

                        // increment cursor
                        self.cursor.position += caps_len;

                        for c in text.chars() {
                            track_position_by_char!(self.cursor, c);
                        }

                        return self.next();
                    }
                }
            }
        }

        match v {
            Some(r) => Some(r),
            None => return Some(Err(LexerError::NoMatch)),
        }
    }

    pub fn peek(&mut self) -> Option<Result<MatchedToken<T>, LexerError>> {
        if self.is_eof() {
            return None;
        }

        // remember current position
        let current_cursor = self.cursor.clone();

        // get next token
        let token = self.next();

        // restore position
        self.cursor = current_cursor;

        // return token
        token
    }

    pub fn skip(&mut self) -> Result<(), LexerError> {
        match self.next() {
            Some(Ok(_)) => Ok(()),
            Some(Err(e)) => Err(e),
            None => Err(LexerError::EndOfFile),
        }
    }

    pub fn is_eof(&self) -> bool {
        self.cursor.position >= self.source.len()
    }

    pub fn has_more_tokens(&mut self) -> bool {
        match self.peek() {
            Some(Ok(_)) => true,
            Some(Err(_)) => false,
            None => false,
        }
    }

    pub fn get_cursor(&self) -> Cursor {
        self.cursor.clone()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    enum MyTokens {
        Mary,
        Lamb,
        Numeric,
        Text,
        Baa,
        Function,
        End,
        Comment,
        //MultlineComment,
        NoArgs,
        Identifier,
    }

    #[test]
    fn test_mary_had_a_little_lamb() {
        let tokens = vec![
            match_token!("Mary", MyTokens::Mary),
            match_token!("lamb", MyTokens::Lamb),
            match_regex!("^[0-9]+", MyTokens::Numeric),
            match_regex!("^[a-zA-Z]+", MyTokens::Text),
        ];

        let mut t = Lexer::new("Mary had a little lamb", tokens, LexerOptions { ignore_whitespace: true });

        let expected = MatchedToken { token: MyTokens::Mary, text: String::from("Mary"), cursor: Cursor { position: 0, line: 1, column: 1 } };
        assert_eq!(t.next(), Some(Ok(expected)), "Expected Mary");
        assert!(t.has_more_tokens(), "Not expected EOF");

        let expected = MatchedToken { token: MyTokens::Text, text: String::from("had"), cursor: Cursor { position: 5, line: 1, column: 6 } };
        assert_eq!(t.next(), Some(Ok(expected)), "Expected had");
        assert!(t.has_more_tokens(), "Not expected EOF");

        let expected = MatchedToken { token: MyTokens::Text, text: String::from("a"), cursor: Cursor { position: 9, line: 1, column: 10 } };
        assert_eq!(t.next(), Some(Ok(expected)), "Expected a");
        assert!(t.has_more_tokens(), "Not expected EOF");

        let expected = MatchedToken { token: MyTokens::Text, text: String::from("little"), cursor: Cursor { position: 11, line: 1, column: 12 } };
        assert_eq!(t.next(), Some(Ok(expected)), "Expected little");
        assert!(t.has_more_tokens(), "Not expected EOF");

        let expected = MatchedToken { token: MyTokens::Lamb, text: String::from("lamb"), cursor: Cursor { position: 18, line: 1, column: 19 } };
        assert_eq!(t.next(), Some(Ok(expected)), "Expected lamb");

        assert!(!t.has_more_tokens(), "No more tokens expected");
        assert!(t.is_eof(), "Expected EOF");
    }

    #[test]
    fn test_peek_token() {
        let tokens = vec![
            match_regex!("^[a-zA-Z]+", MyTokens::Text),
        ];

        let mut t = Lexer::new("Merry Christmas", tokens, LexerOptions { ignore_whitespace: true });

        let expected = MatchedToken { token: MyTokens::Text, text: String::from("Merry"), cursor: Cursor { position: 0, line: 1, column: 1 } };
        assert_eq!(t.peek(), Some(Ok(expected.clone())), "Expected Merry");
        assert_eq!(t.next(), Some(Ok(expected.clone())), "Expected Merry");
        assert!(t.has_more_tokens(), "Not expected EOF");

        let expected = MatchedToken { token: MyTokens::Text, text: String::from("Christmas"), cursor: Cursor { position: 6, line: 1, column: 7 } };
        assert_eq!(t.peek(), Some(Ok(expected.clone())), "Expected Christmas");
        assert_eq!(t.next(), Some(Ok(expected.clone())), "Expected Christmas");

        assert!(!t.has_more_tokens(), "No more tokens expected");
        assert!(t.is_eof(), "Expected EOF");
    }

    #[test]
    fn test_baa_baa() {
        let tokens = vec![
            match_token!("Baa", MyTokens::Baa, false),
            match_regex!("^[a-zA-Z]+", MyTokens::Text),
        ];

        let mut t = Lexer::new("Baa baa black sheep", tokens, LexerOptions { ignore_whitespace: true });

        let expected = MatchedToken { token: MyTokens::Baa, text: String::from("Baa"), cursor: Cursor { position: 0, line: 1, column: 1 } };
        assert_eq!(t.next(), Some(Ok(expected)), "Expected Baa");

        let expected = MatchedToken { token: MyTokens::Baa, text: String::from("baa"), cursor: Cursor { position: 4, line: 1, column: 5 } };
        assert_eq!(t.next(), Some(Ok(expected)), "Expected baa");

        let expected = MatchedToken { token: MyTokens::Text, text: String::from("black"), cursor: Cursor { position: 8, line: 1, column: 9 } };
        assert_eq!(t.next(), Some(Ok(expected)), "Expected black");

        let expected = MatchedToken { token: MyTokens::Text, text: String::from("sheep"), cursor: Cursor { position: 14, line: 1, column: 15 } };
        assert_eq!(t.next(), Some(Ok(expected)), "Expected sheep");

        assert!(!t.has_more_tokens(), "No more tokens expected");
        assert!(t.is_eof(), "Expected EOF");
    }

    #[test]
    fn test_function_declaration() {
        let tokens = vec![
            Matcher::MatchToken { value: MyTokens::Function, condition: String::from("function"), case_sensitive: true },
            Matcher::MatchToken { value: MyTokens::End, condition: String::from("end"), case_sensitive: false },
            Matcher::MatchRegex { value: MyTokens::Comment, condition: String::from(r"^--[^\n]*") },
            //Matcher::Regex { value: MyTokens::MultlineComment, condition: String::from(r"^--\[\[[^\]\]]*\]\]") },
            Matcher::MatchToken { value: MyTokens::NoArgs, condition: String::from("()"), case_sensitive: false },
            Matcher::MatchRegex { value: MyTokens::Identifier, condition: String::from(r"^[a-zA-Z_][a-zA-Z0-9_]*") },
        ];

        let mut t = Lexer::new(r#"
        -- this is a comment
        function main()
        end
        "#, tokens, LexerOptions { ignore_whitespace: true });

        let expected = MatchedToken { token: MyTokens::Comment, text: String::from("-- this is a comment"), cursor: Cursor { position: 9, line: 2, column: 9 } };
        assert_eq!(t.next(), Some(Ok(expected)), "Expected comment");

        let expected = MatchedToken { token: MyTokens::Function, text: String::from("function"), cursor: Cursor { position: 38, line: 3, column: 9 } };
        assert_eq!(t.next(), Some(Ok(expected)), "Expected function");

        let expected = MatchedToken { token: MyTokens::Identifier, text: String::from("main"), cursor: Cursor { position: 47, line: 3, column: 18 } };
        assert_eq!(t.next(), Some(Ok(expected)), "Expected main");

        let expected = MatchedToken { token: MyTokens::NoArgs, text: String::from("()"), cursor: Cursor { position: 51, line: 3, column: 22 } };
        assert_eq!(t.next(), Some(Ok(expected)), "Expected ()");

        // let expected = MatchedToken { token: MyTokens::MultlineComment, text: String::from("--[[\n        this is a multiline comment\n        ]]"), cursor: Cursor { position: 55, line: 4, column: 9 } };
        // assert_eq!(t.next(), Some(Ok(expected)), "Expected multiline comment");

        let exepected = MatchedToken { token: MyTokens::End, text: String::from("end"), cursor: Cursor { position: 62, line: 4, column: 9 } };
        assert_eq!(t.next(), Some(Ok(exepected)), "Expected end");

        assert!(!t.has_more_tokens(), "No more tokens expected");
    }
}