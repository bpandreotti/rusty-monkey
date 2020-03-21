use crate::token::*;

pub struct Lexer {
    chars: Vec<char>,
    position: usize,
    current_char: Option<char>,
}

impl Lexer {
    pub fn new(input: String) -> Lexer {
        let chars: Vec<char> = input.chars().collect();
        // Using `copied` because the input might be empty, in which case we want to copy over the
        // `None` returned from `get`.
        let current_char = chars.get(0).copied();

        Lexer {
            chars,
            position: 0,
            current_char,
        }
    }

    pub fn next_token(&mut self) -> Token {
        self.consume_whitespace();
        let tok = match self.current_char {
            // Operators
            // Two character operators (==, !=, <=, >=)
            Some('=') if self.peek_char() == Some('=') => { self.read_char(); Token::Equals },
            Some('!') if self.peek_char() == Some('=') => { self.read_char(); Token::NotEquals },
            Some('<') if self.peek_char() == Some('=') => { self.read_char(); Token::LessEq },
            Some('>') if self.peek_char() == Some('=') => { self.read_char(); Token::GreaterEq },
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
            Some('(') => Token::OpenParen,
            Some(')') => Token::CloseParen,
            Some('{') => Token::OpenBrace,
            Some('}') => Token::CloseBrace,

            // Early exit, because we don't need to `read_char()` after the match block.
            Some(c) if c.is_ascii_digit() => return self.read_number(),
            // Identifiers can have any alphanumeric character, but they can't begin with an ascii
            // digit, in which case it will be interpreted as a number.
            Some(c) if c.is_alphanumeric() => return self.read_identifier(),

            Some(c) => Token::Illegal(c),
            None => Token::EOF,
        };
        self.read_char();

        tok
    }

    fn read_char(&mut self) {
        self.position += 1;
        self.current_char = self.chars.get(self.position).copied();
    }

    fn peek_char(&self) -> Option<char> {
        self.chars.get(self.position + 1).copied()
    }

    fn read_identifier(&mut self) -> Token {
        let mut literal = String::new();
        while let Some(ch) = self.current_char {
            if !ch.is_alphanumeric() { break; }
            literal.push(ch);
            self.read_char();
        }

        // Checks if the literal matches any keyword. If it doesn't, it's an identifier.
        Lexer::match_keyword(&literal).unwrap_or(Token::Identifier(literal))
    }

    fn read_number(&mut self) -> Token {
        let mut literal = String::new();
        while let Some(ch) = self.current_char {
            if !ch.is_ascii_digit() { break; }
            literal.push(ch);
            self.read_char();
        }

        Token::Int(literal.parse().unwrap())
    }

    fn consume_whitespace(&mut self) {
        while let Some(ch) = self.current_char {
            if !ch.is_whitespace() { break; }
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
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::Token;

    #[test]
    fn test_next_token() {
        use Token::*;

        // Shortcut to create a `Token::Identifier` from a string literal.
        macro_rules! iden {
            ($x:expr) => { Token::Identifier($x.into()) }
        }

        let input: String = r#"
            let five = 5;
            let add = fn(x, y) {
                x + y;
            };

            !-/*5;

            if (5 < 10) {
                return true;
            } else {
                return false;
            }

            10 == 10;
            != <= >= ?
        "#
        .into();

        let expected = [
            Let,        iden!("five"),          Assign,     Int(5),     Semicolon,  Let,
            iden!("add"),           Assign,     Function,   OpenParen,  iden!("x"), Comma,
            iden!("y"), CloseParen, OpenBrace,  iden!("x"), Plus,       iden!("y"), Semicolon,
            CloseBrace, Semicolon,  Bang,       Minus,      Slash,      Asterisk,   Int(5),
            Semicolon,  If,         OpenParen,  Int(5),     LessThan,   Int(10),    CloseParen,
            OpenBrace,  Return,     True,       Semicolon,  CloseBrace, Else,       OpenBrace,
            Return,     False,      Semicolon,  CloseBrace, Int(10),    Equals,     Int(10),
            Semicolon,  NotEquals,  LessEq,     GreaterEq,  Illegal('?'),           EOF,
        ];

        let mut lex = Lexer::new(input);
        let mut got = lex.next_token();
        for expected_token in expected.iter() {
            assert_eq!(&got, expected_token);
            got = lex.next_token();
        }

        assert_eq!(got, Token::EOF);
    }
}
