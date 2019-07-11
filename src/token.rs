#[derive(PartialEq, Debug)]
pub enum Token {
    Illegal(char),
    EOF,
    Identifier(String),
    Int(i64),
    Assign,
    Plus,
    Comma,
    Semicolon,
    OpenParen,
    CloseParen,
    OpenBrace,
    CloseBrace,
    Function,
    Let,
}
