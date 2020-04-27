// @TODO: Document this module
use crate::error::*;
use crate::token::*;

use std::io::{BufRead, BufReader, Cursor, self};
use std::iter::Peekable;

type LexerLine = Peekable<std::vec::IntoIter<(usize, char)>>;
type LexerLines = Box<dyn Iterator<Item = (usize, Result<LexerLine, io::Error>)>>;

pub struct Lexer {
    lines: Peekable<LexerLines>,
    // `token_position` is the position of the first character of the last token returned by
    // `Lexer::next_token`, and `current_position` is the position of `current_char`.
    pub token_position: (usize, usize),
    current_position: (usize, usize),
    current_char: Option<char>,
    current_line: Option<LexerLine>,
}

impl Lexer {
    pub fn new(input: Box<dyn BufRead>) -> Result<Lexer, io::Error> {
        let lines = input
            .lines()
            .map(|line_result| {
                line_result.map(|line| {
                    line.chars()
                        .chain(std::iter::once('\n'))
                        .enumerate()
                        .collect::<Vec<_>>()
                        .into_iter()
                        .peekable()
                })
            })
            .enumerate();
        let lines = (Box::new(lines) as LexerLines).peekable();

        let mut lex = Lexer {
            lines,
            token_position: (0, 0),
            current_position: (0, 0),
            current_line: None,
            current_char: None,
        };
        lex.read_line()?;
        lex.read_char()?;
        Ok(lex)
    }

    pub fn from_string(input: String) -> Result<Lexer, io::Error> {
        let cursor = Cursor::new(input.into_bytes());
        Lexer::new(Box::new(BufReader::new(cursor)))
    }

    fn read_line(&mut self) -> Result<(), io::Error> {
        match self.lines.next() {
            Some((number, line_result)) => {
                self.current_position.0 = number + 1;
                self.current_line = Some(line_result?);
            }
            None => self.current_line = None,
        }
        Ok(())
    }

    fn read_char(&mut self) -> Result<(), io::Error> {
        match &mut self.current_line {
            Some(line) => match line.next() {
                Some((number, character)) => {
                    self.current_position.1 = number + 1;
                    self.current_char = Some(character);
                }
                None => {
                    self.read_line()?;
                    return self.read_char();
                }
            },
            None => self.current_char = None,
        };
        Ok(())
    }

    fn peek_char(&mut self) -> Option<char> {
        // Because `Peekable::peek` returns an immutable referece but takes a mutable one, I can't
        // use `peek` to get the next line, and then use `peek` again on it to get its first
        // character. Because of that, if we are on a line boundary -- that is, if
        // `current_char` is the last character in the current line -- `peek_char` will return
        // `None`.
        self.current_line.as_mut().and_then(|l| l.peek().map(|c| c.1))
    }

    pub fn next_token(&mut self) -> Result<Token, MonkeyError> {
        self.consume_whitespace()?;
        self.token_position = self.current_position;
        let peek_ch = self.peek_char();
        let tok = match self.current_char {
            // Comments
            Some('/') if peek_ch == Some('/') => {
                self.read_line()?;
                self.read_char()?;
                return self.next_token();
            }

            // Operators
            // Two character operators (==, !=, <=, >=)
            Some('=') if peek_ch == Some('=') => { self.read_char()?; Token::Equals }
            Some('!') if peek_ch == Some('=') => { self.read_char()?; Token::NotEquals }
            Some('<') if peek_ch == Some('=') => { self.read_char()?; Token::LessEq }
            Some('>') if peek_ch == Some('=') => { self.read_char()?; Token::GreaterEq }
            // Single character operators
            Some('=') => Token::Assign,
            Some('!') => Token::Bang,
            Some('+') => Token::Plus,
            Some('-') => Token::Minus,
            Some('*') => Token::Asterisk,
            Some('/') => Token::Slash,
            Some('^') => Token::Exponent,
            Some('%') => Token::Modulo,
            Some('<') => Token::LessThan,
            Some('>') => Token::GreaterThan,

            // Delimiters
            Some(',') => Token::Comma,
            Some(';') => Token::Semicolon,
            Some(':') => Token::Colon,
            Some('(') => Token::OpenParen,
            Some(')') => Token::CloseParen,
            Some('{') => Token::OpenCurlyBrace,
            Some('}') => Token::CloseCurlyBrace,
            Some('[') => Token::OpenSquareBracket,
            Some(']') => Token::CloseSquareBracket,
            Some('#') if peek_ch == Some('{') => { self.read_char()?; Token::OpenHash }

            Some('\"') => self.read_string()?,

            // Early exit, because we don't need to `read_char()` after the match block
            Some(c) if c.is_ascii_digit() => return self.read_number(),
            // Identifiers can have any alphanumeric character, but they can't begin with an ascii
            // digit, in which case it will be interpreted as a number
            Some(c) if c.is_alphanumeric() || c == '_' => return self.read_identifier(),

            Some(c) => return Err(lexer_err(self.current_position, LexerError::IllegalChar(c))),
            None => return Ok(Token::EOF),
        };
        self.read_char()?;
        Ok(tok)
    }

    fn read_identifier(&mut self) -> Result<Token, MonkeyError> {
        let mut literal = String::new();
        while let Some(ch) = self.current_char {
            if !ch.is_alphanumeric() && ch != '_' {
                break;
            }
            literal.push(ch);
            self.read_char()?;
        }

        // Checks if the literal matches any keyword. If it doesn't, it's an identifier
        Ok(Lexer::match_keyword(&literal).unwrap_or(Token::Identifier(literal)))
    }

    fn read_number(&mut self) -> Result<Token, MonkeyError> {
        let mut literal = String::new();
        while let Some(ch) = self.current_char {
            if ch.is_ascii_digit() {
                literal.push(ch);
            } else if ch != '_' {
                break;
            }
            self.read_char()?;
        }

        Ok(Token::Int(literal.parse().unwrap()))
    }

    fn read_string(&mut self) -> Result<Token, MonkeyError> {
        let mut result = String::new();
        loop {
            self.read_char()?;
            match self.current_char {
                Some('"') => break,
                Some('\\') => {
                    self.read_char()?;
                    match self.current_char {
                        Some('\\') => result.push('\\'),
                        Some('n') => result.push('\n'),
                        Some('t') => result.push('\t'),
                        Some('r') => result.push('\r'),
                        Some('"') => result.push('"'),
                        Some(c) => return Err(lexer_err(
                            self.current_position,
                            LexerError::UnknownEscapeSequence(c)
                        )),
                        None => return Err(lexer_err(
                            self.current_position,
                            LexerError::UnexpectedEOF
                        )),
                    }
                }
                Some(c) => result.push(c),
                None => return Err(lexer_err(
                    self.current_position,
                    LexerError::UnexpectedEOF
                )),
            }
        }
        Ok(Token::Str(result))
    }

    fn consume_whitespace(&mut self) -> Result<(), io::Error> {
        while let Some(ch) = self.current_char {
            if !ch.is_whitespace() {
                break;
            }
            self.read_char()?;
        }
        Ok(())
    }

    fn match_keyword(literal: &str) -> Option<Token> {
        match literal {
            "fn" => Some(Token::Function),
            "let" => Some(Token::Let),
            "true" => Some(Token::True),
            "false" => Some(Token::False),
            "if" => Some(Token::If),
            "else" => Some(Token::Else),
            "return" => Some(Token::Return),
            "nil" => Some(Token::Nil),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::Token;

    // Shortcut to create a `Token::Identifier` from a string literal
    macro_rules! iden {
        ($x:expr) => { Token::Identifier($x.into()) }
    }

    fn assert_lex(input: &str, expected: &[Token]) {
        let mut lex = Lexer::from_string(input.into()).unwrap();
        for ex in expected {
            let got = lex.next_token().expect("Lexer error during test");
            assert_eq!(ex, &got);
        }
    }

    fn assert_lexer_error(input: &str, expected_error: LexerError) {
        let mut lex = Lexer::from_string(input.into()).unwrap();
        loop {
            match lex.next_token() {
                Ok(Token::EOF) => panic!("No lexer errors encountered"),
                Err(e) => {
                    match e.error {
                        ErrorType::Lexer(got) => assert_eq!(expected_error, got),
                        _ => panic!("Wrong error type")
                    }
                    return;
                }
                _ => continue,
            }
        }
    }

    #[test]
    fn test_identifiers() {
        let input = "foo bar two_words _ _foo2 back2thefuture 3different_ones olá 統一碼 यूनिकोड";
        let expected = [
            iden!("foo"),
            iden!("bar"),
            iden!("two_words"),
            iden!("_"),
            iden!("_foo2"),
            iden!("back2thefuture"),
            Token::Int(3),
            iden!("different_ones"),
            iden!("olá"),
            iden!("統一碼"),
            iden!("यूनिकोड"),
            Token::EOF,
        ];
        assert_lex(input, &expected);

        // Test keywords
        let input = "fn let true false if else return nil";
        let expected = [
            Token::Function,
            Token::Let,
            Token::True,
            Token::False,
            Token::If,
            Token::Else,
            Token::Return,
            Token::Nil,
            Token::EOF,
        ];
        assert_lex(input, &expected);
    }

    #[test]
    fn test_int_literals() {
        let input = "0 1729 808017424794 1_000_000 1___0____2 _1_000_000";
        let expected = [
            Token::Int(0),
            Token::Int(1729),
            Token::Int(808_017_424_794),
            Token::Int(1_000_000),
            Token::Int(102),
            iden!("_1_000_000"),
            Token::EOF,
        ];
        assert_lex(input, &expected);
    }

    #[test]
    fn test_strings() {
        let input = r#"
            "string"
            "escape sequences: \\ \n \t \r \" "
            "whitespace
            inside strings"
        "#;
        let expected = [
            Token::Str("string".into()),
            Token::Str("escape sequences: \\ \n \t \r \" ".into()),
            Token::Str("whitespace\n            inside strings".into()),
            Token::EOF,
        ];
        assert_lex(input, &expected);
    }

    #[test]
    fn test_operators() {
        let input = "= ! + - * / ^ % < > == != <= >=";
        let expected = [
            Token::Assign,
            Token::Bang,
            Token::Plus,
            Token::Minus,
            Token::Asterisk,
            Token::Slash,
            Token::Exponent,
            Token::Modulo,
            Token::LessThan,
            Token::GreaterThan,
            Token::Equals,
            Token::NotEquals,
            Token::LessEq,
            Token::GreaterEq,
            Token::EOF,
        ];
        assert_lex(input, &expected);
    }

    #[test]
    fn test_delimiters() {
        let input = ", ; : () {} [] #{}";
        let expected = [
            Token::Comma,
            Token::Semicolon,
            Token::Colon,
            Token::OpenParen,
            Token::CloseParen,
            Token::OpenCurlyBrace,
            Token::CloseCurlyBrace,
            Token::OpenSquareBracket,
            Token::CloseSquareBracket,
            Token::OpenHash,
            Token::CloseCurlyBrace,
            Token::EOF,
        ];
        assert_lex(input, &expected);
    }

    #[test]
    fn test_comments() {
        let input = r"
            // comments
            foo // bar
            // Unicode! 中文 Português हिन्दी Français Español
            //
            baz
        ";
        let expected = [iden!("foo"), iden!("baz"), Token::EOF];
        assert_lex(input, &expected);
    }

    #[test]
    fn test_large_program() {
        use Token::*;
        let input = r#"
            let fizzbuzz = fn(i) {
                let result = if i % 3 == 0 {
                    "fizz"
                } else {
                    ""
                };
                let result = if i % 5 == 0 {
                    result + "buzz"
                } else {
                    result
                };
                if result != "" {
                    puts(result)
                } else {
                    puts(i)
                }
            }
            map(fizzbuzz, range(1, 100));
        "#;
        let expected = [
            // Line 1
            Let,
            iden!("fizzbuzz"),
            Assign,
            Function,
            OpenParen,
            iden!("i"),
            CloseParen,
            OpenCurlyBrace,

            // Line 2
            Let,
            iden!("result"),
            Assign,
            If,
            iden!("i"),
            Modulo,
            Int(3),
            Equals,
            Int(0),
            OpenCurlyBrace,

            // Line 3
            Str("fizz".into()),

            // Line 4
            CloseCurlyBrace,
            Else,
            OpenCurlyBrace,

            // Line 5
            Str("".into()),

            // Line 6
            CloseCurlyBrace,
            Semicolon,

            // Line 7
            Let,
            iden!("result"),
            Assign,
            If,
            iden!("i"),
            Modulo,
            Int(5),
            Equals,
            Int(0),
            OpenCurlyBrace,

            // Line 8
            iden!("result"),
            Plus,
            Str("buzz".into()),

            // Line 9
            CloseCurlyBrace,
            Else,
            OpenCurlyBrace,

            // Line 10
            iden!("result"),

            // Line 11
            CloseCurlyBrace,
            Semicolon,

            // Line 12
            If,
            iden!("result"),
            NotEquals,
            Str("".into()),
            OpenCurlyBrace,

            // Line 13
            iden!("puts"),
            OpenParen,
            iden!("result"),
            CloseParen,

            // Line 14
            CloseCurlyBrace,
            Else,
            OpenCurlyBrace,

            // Line 15
            iden!("puts"),
            OpenParen,
            iden!("i"),
            CloseParen,

            // Line 16
            CloseCurlyBrace,
            
            // Line 17
            CloseCurlyBrace,
            
            // Line 18
            iden!("map"),
            OpenParen,
            iden!("fizzbuzz"),
            Comma,
            iden!("range"),
            OpenParen,
            Int(1),
            Comma,
            Int(100),
            CloseParen,
            CloseParen,
            Semicolon,
            EOF,
        ];
        assert_lex(input, &expected);
    }

    #[test]
    fn test_lexer_position() {
        let program = "first line\nsecond\n3";
        let expected = [
            // (token, current_position, token_position)
            (iden!("first"), (1, 6), (1, 1)),
            (iden!("line"), (1, 11), (1, 7)),
            (iden!("second"), (2, 7), (2, 1)),
            (Token::Int(3), (3, 2), (3, 1)),
            (Token::EOF, (3, 2), (3, 2)),
        ];

        let mut lex = Lexer::from_string(program.into()).unwrap();
        assert_eq!((1, 1), lex.current_position);
        assert_eq!((0, 0), lex.token_position);

        for (tk, current_pos, token_pos) in &expected {
            let got = lex.next_token().unwrap();
            assert_eq!(tk, &got);
            assert_eq!(current_pos, &lex.current_position);
            assert_eq!(token_pos, &lex.token_position);
        }
    }

    #[test]
    fn test_lexer_errors() {
        assert_lexer_error(
            r#" "some string that doesn't end "#,
            LexerError::UnexpectedEOF
        );
        assert_lexer_error(
            r#" "i don't know this guy: \w" "#,
            LexerError::UnknownEscapeSequence('w')
        );
        assert_lexer_error(
            r#" "whats up with this weird symbol:" & "#,
            LexerError::IllegalChar('&')
        );
    }
}
