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
}