#[derive(PartialEq, Debug)]
pub enum Token {
    Illegal(char),
    EOF,

    Identifier(String),
    Int(i64),

    // Operators
    Assign,
    Bang,
    Plus,
    Minus,
    Asterisk,
    Slash,
    LessThan,
    GreaterThan,

    // Delimiters
    Comma,
    Semicolon,
    OpenParen,
    CloseParen,
    OpenBrace,
    CloseBrace,
    
    // Keywords
    Function,
    Let,
    True,
    False,
    If,
    Else,
    Return,
}
