// @WIP
use crate::object::Object;
use crate::token::Token;

use std::fmt;

pub struct MonkeyError {
    pub position: (usize, usize),
    pub error: ErrorType,
}

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

pub enum ErrorType {
    Lexer(LexerError),
    Parser(ParserError),
    Runtime(RuntimeError),
}

pub enum LexerError {

}

pub enum ParserError {
    
}

pub enum RuntimeError {
    // @TODO: Maybe instead of storing `Object`s, we should just store their `type_str`s
    // Identifier not found in the current environment
    IdenNotFound(String),
    // Return outside of function context
    InvalidReturn,
    // Trying to call a function with the wrong number of arguments
    WrongNumberOfArgs(usize, usize), // @TODO: Maybe change this to allow for variadic functions
    // Trying to index array using non-integer index
    ArrayIndexTypeError(Object),
    // Array index out of bounds
    IndexOutOfBounds(i64),
    // Trying to index hash using non-hashable key type
    HashKeyTypeError(Object),
    // Value not found in hash
    KeyError(Object),
    // Trying to index an object which is not an array or a hash
    IndexingWrongType(Object),
    // Invalid type in prefix expression
    PrefixTypeError(Token, Object),
    // Invalid type in infix expression
    InfixTypeError(Object, Token, Object),
    // Trying to call non-callable object
    NotCallable(Object),
}

impl RuntimeError {
    pub fn message(&self) -> String {
        match self {
            RuntimeError::IdenNotFound(s) => format!("identifier not found: {}", s),
            RuntimeError::InvalidReturn => "`return` outside of function context".to_string(),
            RuntimeError::WrongNumberOfArgs(expected, got) => format!(
                "wrong number of arguments: expected {} arguments but {} were given",
                expected,
                got
            ),
            RuntimeError::ArrayIndexTypeError(o) => format!(
                "array index must be integer, not '{}'",
                o.type_str()
            ),
            RuntimeError::IndexOutOfBounds(i) => format!("array index out of bounds: {}", i),
            RuntimeError::HashKeyTypeError(o) => format!(
                "hash key must be hashable type, not '{}'",
                o.type_str()
            ),
            RuntimeError::KeyError(o) => format!(
                "hash key error: entry for {} not found",
                o
            ),
            RuntimeError::IndexingWrongType(o) => format!(
                "'{}' is not an array or hash object",
                o.type_str()
            ),
            RuntimeError::PrefixTypeError(tk, o) => format!(
                "unsuported operand type for prefix operator {}: '{}'",
                tk.type_str(),
                o.type_str()
            ),
            RuntimeError::InfixTypeError(l, tk, r) => format!(
                "unsuported operand types for infix operator {}: '{}' and '{}'",
                tk.type_str(),
                l.type_str(),
                r.type_str()
            ),
            RuntimeError::NotCallable(o) => format!(
                "'{}' is not a function object",
                o.type_str()
            ),
        }
    }
}
