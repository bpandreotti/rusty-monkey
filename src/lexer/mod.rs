// @TODO: Document this module
#[cfg(test)]
mod tests;
pub mod token;

use crate::error::*;
use token::Token;

use std::io::{self, BufRead, BufReader, Cursor};
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
        self.current_line
            .as_mut()
            .and_then(|l| l.peek().map(|c| c.1))
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
            Some('=') if peek_ch == Some('=') => {
                self.read_char()?;
                Token::Equals
            }
            Some('!') if peek_ch == Some('=') => {
                self.read_char()?;
                Token::NotEquals
            }
            Some('<') if peek_ch == Some('=') => {
                self.read_char()?;
                Token::LessEq
            }
            Some('>') if peek_ch == Some('=') => {
                self.read_char()?;
                Token::GreaterEq
            }
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
            Some('#') if peek_ch == Some('{') => {
                self.read_char()?;
                Token::OpenHash
            }

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
                        Some(c) => {
                            return Err(lexer_err(
                                self.current_position,
                                LexerError::UnknownEscapeSequence(c),
                            ))
                        }
                        None => {
                            return Err(lexer_err(self.current_position, LexerError::UnexpectedEOF))
                        }
                    }
                }
                Some(c) => result.push(c),
                None => return Err(lexer_err(self.current_position, LexerError::UnexpectedEOF)),
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
