use std::fmt;

#[derive(PartialEq, Debug, Clone)]
pub enum Token {
    Illegal(char),
    EOF,

    Identifier(String),
    Int(i64),
    Str(String),

    // Operators
    Assign,
    Bang,
    Plus,
    Minus,
    Asterisk,
    Slash,
    LessThan,
    GreaterThan,
    Equals,
    NotEquals,
    LessEq,
    GreaterEq,

    // Delimiters
    Comma,
    Semicolon,
    Colon,
    OpenParen,
    CloseParen,
    OpenCurlyBrace,
    CloseCurlyBrace,
    OpenSquareBracket,
    CloseSquareBracket,
    OpenHash,

    // Keywords
    Function,
    Let,
    True,
    False,
    If,
    Else,
    Return,
    Nil,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.type_str())
    }
}

impl Token {
    /// Returns a string representing the token type, i.e., the enum variant.
    fn type_str(&self) -> &'static str {
        use Token::*; // So the big-ass table doesn't need to have "Token::" everywhere.
        match self {
            Identifier(_) => "identifier",
            Int(_) => "integer literal",
            Str(_) => "string literal",
            Assign => "`=`",
            Bang => "`!`",
            Plus => "`+`",
            Minus => "`-`",
            Asterisk => "`*`",
            Slash => "`/`",
            LessThan => "`<`",
            GreaterThan => "`>`",
            Equals => "`==`",
            NotEquals => "`!=`",
            LessEq => "`<=`",
            GreaterEq => "`>=`",
            Comma => "`,`",
            Semicolon => "`;`",
            Colon => "`:`",
            OpenParen => "`(`",
            CloseParen => "`)`",
            OpenCurlyBrace => "`{`",
            CloseCurlyBrace => "`}`",
            OpenSquareBracket => "`[`",
            CloseSquareBracket => "`[`",
            OpenHash => "`#{`",
            Function => "`fn`",
            Let => "`let`",
            True | False => "boolean literal",
            If => "`if`",
            Else => "`else`",
            Return => "`return`",
            Illegal(_) => "illegal",
            EOF => "EOF",
            Nil => "`nil`",
        }
    }
}
