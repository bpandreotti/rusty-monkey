// @WIP
use crate::object::*;
use crate::token::Token;

use std::fmt;

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
            ErrorType::Runtime(e) => write!(f, "Runtime error: {}", e.message()),
            _ => todo!(), // @TODO: Implement Lexer and Parser errors
        }
    }
}

impl std::convert::From<std::io::Error> for MonkeyError {
    fn from(error: std::io::Error) -> MonkeyError {
        MonkeyError {
            position: (0, 0), // If it's an IO error, the position doesn't really matter
            error: ErrorType::Lexer(LexerError::IoError(error)),
        }
    }
}


#[derive(Debug)]
pub enum ErrorType {
    Lexer(LexerError),
    Parser(ParserError),
    Runtime(RuntimeError),
}

#[derive(Debug)]
pub enum LexerError {
    IoError(std::io::Error),
    UnexpectedEOF,
    UnknownEscapeSequence(char),
    IllegalChar(char),
}

#[derive(Debug)]
pub enum ParserError {}

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
    // Division by zero
    DivisionByZero,
    // Custom error
    Custom(String),
}

impl RuntimeError {
    pub fn message(&self) -> String {
        match self {
            RuntimeError::IdenNotFound(s) => format!("identifier not found: '{}'", s),
            RuntimeError::InvalidReturn => "`return` outside of function context".to_string(),
            RuntimeError::WrongNumberOfArgs(expected, got) => format!(
                "wrong number of arguments: expected {} arguments but {} were given",
                expected,
                got
            ),
            RuntimeError::ArrayIndexTypeError(obj_type) => format!(
                "array index must be integer, not '{}'",
                obj_type
            ),
            RuntimeError::IndexOutOfBounds(i) => format!("array index out of bounds: {}", i),
            RuntimeError::HashKeyTypeError(obj_type) => format!(
                "hash key must be hashable type, not '{}'",
                obj_type
            ),
            RuntimeError::KeyError(o) => format!(
                "hash key error: entry for {} not found",
                o
            ),
            RuntimeError::IndexingWrongType(obj_type) => format!(
                "'{}' is not an array or hash object",
                obj_type
            ),
            RuntimeError::PrefixTypeError(tk, obj_type) => format!(
                "unsuported operand type for prefix operator {}: '{}'",
                tk,
                obj_type
            ),
            RuntimeError::InfixTypeError(l_type, tk, r_type) => format!(
                "unsuported operand types for infix operator {}: '{}' and '{}'",
                tk,
                l_type,
                r_type,
            ),
            RuntimeError::NotCallable(obj_type) => format!(
                "'{}' is not a function object",
                obj_type
            ),
            RuntimeError::DivisionByZero => "division by zero".to_string(),
            RuntimeError::Custom(msg) => msg.clone(),
        }
    }
}
