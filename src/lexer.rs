use crate::token::*;
use std::iter::Peekable;
use std::str::Chars;

pub struct Lexer {
    chars: Vec<char>,
    position: usize,
    pub current_char: Option<char>,
}

impl Lexer {
    pub fn new(input: String) -> Lexer {
        let chars: Vec<char> = input.chars().collect();
        // `copied` turns an `Option<&T>` into an `Option<T>`.
        let current_char = chars.get(0).copied();

        Lexer {
            chars: chars,
            position: 0,
            current_char: current_char,
        }
    }

    pub fn read_char(&mut self) {
        self.position += 1;
        self.current_char = self.chars.get(self.position).copied();
    }

    pub fn next_token(&mut self) -> Token {
        let tok = match self.current_char {
            Some('=') => Token::Assign,
            Some(';') => Token::Semicolon,
            Some('(') => Token::OpenParen,
            Some(')') => Token::CloseParen,
            Some('{') => Token::OpenBrace,
            Some('}') => Token::CloseBrace,
            Some(',') => Token::Comma,
            Some('+') => Token::Plus,
            
            // Early exit, because we don't need to `read_char()` after the match block.
            Some(c) if c.is_alphabetic() => return self.read_identifier(),
            
            // Whitespace and numbers are for now considered illegal.
            Some(c) => Token::Illegal(c),
            None => Token::EOF,
        };
        self.read_char();
        
        tok
    }

    fn read_identifier(&mut self) -> Token {
        let mut literal = String::new();
        while let Some(ch) = self.current_char {
            if ch.is_alphanumeric() {
                literal.push(ch);
            } else {
                break;
            }
            self.read_char();
        }

        match &literal as &str {
            "let" => Token::Let,
            "fn" => Token::Function,
            _ => Token::Identifier(literal),
        }        
    }
}
