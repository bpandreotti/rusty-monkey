// @TODO: Document this module
// @TODO: Add error handling
use crate::token::*;
use std::io::{ BufRead, BufReader, Cursor };
use std::iter::Peekable;

type LexerLine = Peekable<std::vec::IntoIter<char>>;
type LexerLines = Box<dyn Iterator<Item = LexerLine>>;

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
    pub fn new(input: Box<dyn BufRead>) -> Lexer {
        let lines = input
            .lines()
            .map(|l| {
                // @TODO: Change this `unwrap` to proper error handling
                l.unwrap() // Panic on IO error
                    .chars()
                    .collect::<Vec<_>>()
                    .into_iter()
                    .peekable()
            });
        let lines = (Box::new(lines) as LexerLines).peekable();

        let mut lex = Lexer {
            lines,
            token_position: (0, 0),
            current_position: (0, 0),
            current_line: None,
            current_char: None,
        };
        lex.read_line();
        lex.read_char();
        lex
    }

    pub fn from_string(input: String) -> Lexer {
        let cursor = Cursor::new(input.into_bytes());
        Lexer::new(Box::new(BufReader::new(cursor)))
    }

    fn read_line(&mut self) {
        self.current_position.0 += 1;
        self.current_position.1 = 0;
        self.current_line = self.lines.next();
    }

    fn read_char(&mut self) {
        match &mut self.current_line {
            Some(line) => match line.next() {
                Some(c) => {
                    self.current_position.1 += 1;
                    self.current_char = Some(c)
                }
                None => {
                    self.read_line();
                    // This artificial '\n' is necessary to indicate that there is whitespace in
                    // the end of the line, and without it some problems arise. For instance,
                    // every identifier that ends on a line break would continue on in the next
                    // line. So this:
                    //     a + foo
                    //     let b = 3
                    // would result in the following tokens: "a", "+", "foolet", "b", "=", "3"
                    self.current_char = Some('\n');
                }
            }
            None => self.current_char = None,
        };
    }

    fn peek_char(&mut self) -> Option<&char> {
        // Because `Peekable::peek` returns an immutable referece but takes a mutable one, I can't
        // use `peek` to get the next line, and then use `peek` again on it to get its first
        // character. Because of that, if we are on a line boundary -- that is, if
        // `current_char` is the last character in the current line -- `peek_char` will return
        // `None`.
        self.current_line.as_mut().and_then(|l| l.peek())
    }

    pub fn next_token(&mut self) -> Token {
        self.consume_whitespace();
        self.token_position = self.current_position;
        let peek_ch = self.peek_char().cloned();
        let tok = match self.current_char {
            // Comments
            Some('/') if peek_ch == Some('/') => {
                self.read_line();
                self.read_char();
                return self.next_token();
            }

            // Operators
            // Two character operators (==, !=, <=, >=)
            Some('=') if peek_ch == Some('=') => { self.read_char(); Token::Equals }
            Some('!') if peek_ch == Some('=') => { self.read_char(); Token::NotEquals }
            Some('<') if peek_ch == Some('=') => { self.read_char(); Token::LessEq }
            Some('>') if peek_ch == Some('=') => { self.read_char(); Token::GreaterEq }
            // Single character operators
            Some('=') => Token::Assign,
            Some('!') => Token::Bang,
            Some('+') => Token::Plus,
            Some('-') => Token::Minus,
            Some('*') => Token::Asterisk,
            Some('/') => Token::Slash,
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
            Some('#') if peek_ch == Some('{') => { self.read_char(); Token::OpenHash }

            Some('\"') => self.read_string(),

            // Early exit, because we don't need to `read_char()` after the match block
            Some(c) if c.is_ascii_digit() => return self.read_number(),
            // Identifiers can have any alphanumeric character, but they can't begin with an ascii
            // digit, in which case it will be interpreted as a number
            Some(c) if c.is_alphanumeric() => return self.read_identifier(),

            Some(c) => Token::Illegal(c),
            None => Token::EOF,
        };
        self.read_char();
        tok
    }

    fn read_identifier(&mut self) -> Token {
        let mut literal = String::new();
        while let Some(ch) = self.current_char {
            if !ch.is_alphanumeric() && ch != '_' {
                break;
            }
            literal.push(ch);
            self.read_char();
        }

        // Checks if the literal matches any keyword. If it doesn't, it's an identifier
        Lexer::match_keyword(&literal).unwrap_or(Token::Identifier(literal))
    }

    fn read_number(&mut self) -> Token {
        let mut literal = String::new();
        while let Some(ch) = self.current_char {
            if !ch.is_ascii_digit() {
                break;
            }
            literal.push(ch);
            self.read_char();
        }

        Token::Int(literal.parse().unwrap())
    }

    fn read_string(&mut self) -> Token {
        let mut result = String::new();
        loop {
            self.read_char();
            match self.current_char {
                Some('"') => break,
                Some('\\') => {
                    self.read_char();
                    match self.current_char {
                        Some('\\') => result.push('\\'),
                        Some('n') => result.push('\n'),
                        Some('t') => result.push('\t'),
                        Some('r') => result.push('\r'),
                        Some('"') => result.push('"'),
                        Some(c) => panic!("Unknown escape sequence: \\{}", c),
                        None => panic!(), // We should return an error instead of panicking
                    }
                }
                Some(c) => result.push(c),
                None => panic!(), // We should return an error instead of panicking
            }
        }
        Token::Str(result)
    }

    fn consume_whitespace(&mut self) {
        while let Some(ch) = self.current_char {
            if !ch.is_whitespace() {
                break;
            }
            self.read_char();
        }
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
    // @TODO: Add tests for lexer errors
    // @TODO: Add tests for lexer position
    use super::*;
    use crate::token::Token;

    #[test]
    fn test_next_token() {
        // @TODO: Split up this test into several smaller tests
        use Token::*;

        // Shortcut to create a `Token::Identifier` from a string literal
        macro_rules! iden {
            ($x:expr) => { Token::Identifier($x.into()) }
        }

        let input = r#"
            let five = 5; // testing comments
            let add = fn(x, y) {
                x + y;
            };
            // more comments
            !-/*5;

            //
            // a
            // bunch
            // of
            // comments
            //

            if (5 < 10) {
                return true;
            } else {
                return false;
            }

            // yay

            10 == 10;
            != <= >=
            "foobar" ///
            "foo bar"
            "foo\n\"\tbar"
            ? :
            [1, 2, 3]
            #{
        "#
        .to_string();

        let expected = [
            Let,
            iden!("five"),
            Assign,
            Int(5),
            Semicolon,
            Let,
            iden!("add"),
            Assign,
            Function,
            OpenParen,
            iden!("x"),
            Comma,
            iden!("y"),
            CloseParen,
            OpenCurlyBrace,
            iden!("x"),
            Plus,
            iden!("y"),
            Semicolon,
            CloseCurlyBrace,
            Semicolon,
            Bang,
            Minus,
            Slash,
            Asterisk,
            Int(5),
            Semicolon,
            If,
            OpenParen,
            Int(5),
            LessThan,
            Int(10),
            CloseParen,
            OpenCurlyBrace,
            Return,
            True,
            Semicolon,
            CloseCurlyBrace,
            Else,
            OpenCurlyBrace,
            Return,
            False,
            Semicolon,
            CloseCurlyBrace,
            Int(10),
            Equals,
            Int(10),
            Semicolon,
            NotEquals,
            LessEq,
            GreaterEq,
            Str("foobar".into()),
            Str("foo bar".into()),
            Str("foo\n\"\tbar".into()),
            Illegal('?'),
            Colon,
            OpenSquareBracket,
            Int(1),
            Comma,
            Int(2),
            Comma,
            Int(3),
            CloseSquareBracket,
            OpenHash,
            EOF,
        ];

        let mut lex = Lexer::from_string(input);
        let mut got = lex.next_token();
        for expected_token in expected.iter() {
            assert_eq!(&got, expected_token);
            got = lex.next_token();
        }

        assert_eq!(got, Token::EOF);
    }
}
