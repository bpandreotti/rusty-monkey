use crate::object::*;
use crate::token::Token;

use std::fmt;
use std::io;

pub type MonkeyResult<T> = Result<T, MonkeyError>;

#[derive(Debug)]
pub struct MonkeyError {
    pub position: (usize, usize),
    pub error: ErrorType,
}

impl std::error::Error for MonkeyError {}

impl fmt::Display for MonkeyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "At line {}, column {}:", self.position.0, self.position.1)?;
        write!(f, "    ")?; // Indentation
        match &self.error {
            ErrorType::Io(e) => write!(f, "IO error: {}", e),
            ErrorType::Lexer(e) => write!(f, "Lexer error: {}", e.message()),
            ErrorType::Parser(e) => write!(f, "Parser error: {}", e.message()),
            ErrorType::Runtime(e) => write!(f, "Runtime error: {}", e.message()),
        }
    }
}

impl std::convert::From<io::Error> for MonkeyError {
    fn from(error: io::Error) -> MonkeyError {
        MonkeyError {
            position: (0, 0), // If it's an IO error, the position doesn't really matter
            error: ErrorType::Io(error),
        }
    }
}

// Helper function to build a MonkeyError with a LexerError inside
pub fn lexer_err(pos: (usize, usize), error: LexerError) -> MonkeyError {
    MonkeyError {
        position: pos,
        error: ErrorType::Lexer(error)
    }
}

// Helper function to build a MonkeyError with a ParserError inside
pub fn parser_err(pos: (usize, usize), error: ParserError) -> MonkeyError {
    MonkeyError {
        position: pos,
        error: ErrorType::Parser(error)
    }
}

// Helper function to build a MonkeyError with a RuntimeError inside
pub fn runtime_err(pos: (usize, usize), error: RuntimeError) -> MonkeyError {
    MonkeyError {
        position: pos,
        error: ErrorType::Runtime(error)
    }
}

#[derive(Debug)]
pub enum ErrorType {
    Io(io::Error),
    Lexer(LexerError),
    Parser(ParserError),
    Runtime(RuntimeError),
}

// @TODO: Maybe merge LexerError and Parser Error?
#[derive(Debug)]
pub enum LexerError {
    UnexpectedEOF,
    UnknownEscapeSequence(char),
    IllegalChar(char),
}

impl LexerError {
    pub fn message(&self) -> String {
        use LexerError::*;
        match self {
            UnexpectedEOF => "Unexpected EOF".into(),
            UnknownEscapeSequence(ch) => format!("Unknown escape sequence: \\{}", ch),
            IllegalChar(ch) => format!("Illegal character: \\{}", ch),
        }
    }
}

#[derive(Debug)]
pub enum ParserError {
    UnexpectedToken(Token, Token),
    UnexpectedTokenMultiple {
        possibilities: &'static [Token],
        got: Token
    },
    NoPrefixParseFn(Token),    
}

impl ParserError {
    pub fn message(&self) -> String {
        use ParserError::*;
        match self {
            UnexpectedToken(expected, got) => format!("expected {} token, got {}", expected, got),
            UnexpectedTokenMultiple { possibilities, got } => {
                let mut list = String::new();
                let len = possibilities.len();
                // If there is only one possibility, you really shouldn't be using this variant
                assert!(len > 1);
                for tk in 0..len - 2 {
                    list += &format!("{}, ", tk);
                }
                list += &format!("{} or {}", possibilities[len - 2], possibilities[len - 1]);
                format!("expected {} token, got {}", list, got)
            }
            NoPrefixParseFn(tk) => format!("no prefix parse function found for token: {}", tk),
        }
    }
}

#[derive(Debug)]
pub enum RuntimeError {
    // Identifier not found in the current environment
    IdenNotFound(String),
    // Return outside of function context
    InvalidReturn,
    // Trying to call a function with the wrong number of arguments
    WrongNumberOfArgs(usize, usize), // @TODO: Maybe change this to allow for variadic functions
    // Trying to index array using non-integer index
    ArrayIndexTypeError(&'static str),
    // Array index out of bounds
    IndexOutOfBounds(i64),
    // Trying to index hash using non-hashable key type
    HashKeyTypeError(&'static str),
    // Value not found in hash
    KeyError(HashableObject),
    // Trying to index an object which is not an array or a hash
    IndexingWrongType(&'static str),
    // Invalid type in prefix expression
    PrefixTypeError(Token, &'static str),
    // Invalid type in infix expression
    InfixTypeError(&'static str, Token, &'static str),
    // Trying to call non-callable object
    NotCallable(&'static str),
    // Division or modulo by zero
    DivOrModByZero,
    // Exponentiation with negative exponent
    NegativeExponent,
    // General purpose TypeError, useful for type assertions
    TypeError(&'static str, &'static str),
    // Custom error
    Custom(String),

    ReturnValue(Box<Object>), // @TODO: Document this item
}

impl RuntimeError {
    pub fn message(&self) -> String {
        use RuntimeError::*;
        match self {
            IdenNotFound(s) => format!("identifier not found: '{}'", s),
            InvalidReturn => "`return` outside of function context".to_string(),
            WrongNumberOfArgs(expected, got) => format!(
                "wrong number of arguments: expected {} arguments but {} were given",
                expected,
                got
            ),
            ArrayIndexTypeError(obj) => format!("array index must be integer, not '{}'", obj),
            IndexOutOfBounds(i) => format!("array index out of bounds: {}", i),
            HashKeyTypeError(obj) => format!("hash key must be hashable type, not '{}'", obj),
            KeyError(obj) => format!("hash key error: entry for {} not found", obj),
            IndexingWrongType(obj) => format!("'{}' is not an array or hash object", obj),
            PrefixTypeError(tk, obj) => format!(
                "unsuported operand type for prefix operator {}: '{}'",
                tk,
                obj
            ),
            InfixTypeError(left, tk, right) => format!(
                "unsuported operand types for infix operator {}: '{}' and '{}'",
                tk,
                left,
                right,
            ),
            NotCallable(obj) => format!("'{}' is not a function object", obj),
            DivOrModByZero => "division or modulo by zero".to_string(),
            NegativeExponent => "negative exponent".to_string(),
            TypeError(expected, got) => format!(
                "type error: expected '{}', got '{}'",
                expected,
                got,
            ),
            Custom(msg) => msg.clone(),

            ReturnValue(_) => "`return` outside of function context".to_string(),
        }
    }
}
