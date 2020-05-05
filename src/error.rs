use crate::hashable::HashableObject;
use crate::interpreter::object::*;
use crate::lexer::token::Token;

use colored::*;
use std::fmt;
use std::io;

pub type MonkeyResult<T> = Result<T, MonkeyError>;
type Position = (usize, usize);

#[derive(Debug)]
pub enum MonkeyError {
    Io(io::Error),
    Lexer(Position, LexerError),
    Parser(Position, ParserError),
    Interpreter(Position, RuntimeError),
    Vm(RuntimeError),
}

impl std::error::Error for MonkeyError {}

impl fmt::Display for MonkeyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut write_pos = |&(line, column)| writeln!(f, "At line {}, column {}:", line, column);

        match self {
            MonkeyError::Io(e) => write!(f, "{} {}", "IO error:".red().bold(), e),
            MonkeyError::Lexer(pos, e) => {
                write_pos(pos)?;
                write!(f, "{} {}", "Lexer error:".red().bold(), e)
            }
            MonkeyError::Parser(pos, e) => {
                write_pos(pos)?;
                write!(f, "{} {}", "Parser error:".red().bold(), e)
            }
            MonkeyError::Interpreter(pos, e) => {
                write_pos(pos)?;
                write!(f, "{} {}", "Runtime error:".red().bold(), e)
            }
            MonkeyError::Vm(e) => write!(f, "{} {}", "Runtime error:".red().bold(), e),
        }
    }
}

impl std::convert::From<io::Error> for MonkeyError {
    fn from(error: io::Error) -> MonkeyError {
        MonkeyError::Io(error)
    }
}

#[derive(Debug, PartialEq)]
pub enum LexerError {
    UnexpectedEOF,
    UnknownEscapeSequence(char),
    IllegalChar(char),
}

impl fmt::Display for LexerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use LexerError::*;
        match self {
            UnexpectedEOF => write!(f, "Unexpected EOF"),
            UnknownEscapeSequence(ch) => write!(f, "Unknown escape sequence: \\{}", ch),
            IllegalChar(ch) => write!(f, "Illegal character: \\{}", ch),
        }
    }
}

#[derive(Debug)]
pub enum ParserError {
    UnexpectedToken(Token, Token),
    UnexpectedTokenMultiple {
        possibilities: &'static [Token],
        got: Token,
    },
    NoPrefixParseFn(Token),
}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use ParserError::*;
        match self {
            UnexpectedToken(expected, got) => write!(f, "expected {} token, got {}", expected, got),
            UnexpectedTokenMultiple { possibilities, got } => {
                let len = possibilities.len();
                // If there is only one possibility, you really shouldn't be using this variant
                assert!(len > 1);
                write!(f, "expected ")?;
                for tk in 0..len - 2 {
                    write!(f, "{}, ", tk)?;
                }
                write!(
                    f,
                    "{} or {} token, got {}",
                    possibilities[len - 2],
                    possibilities[len - 1],
                    got
                )
            }
            NoPrefixParseFn(tk) => write!(f, "no prefix parse function found for token: {}", tk),
        }
    }
}

#[derive(Debug)]
pub enum RuntimeError {
    // Identifier not found in the current environment
    IdenNotFound(String),
    // Trying to call a function with the wrong number of arguments
    WrongNumberOfArgs(usize, usize), // @TODO: Maybe change this to allow for variadic functions
    // Trying to index array or string using non-integer index
    IndexTypeError(&'static str),
    // Array or string index out of bounds
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

    StackOverflow,
    StackUnderflow,

    // This error is created whenever the interpreter encounters a return statement. We model
    // returning values this way to take advantage of the error forwarding already present in the
    // evaluator. When the interpreter encounters a runtime error, be it this one or any other,
    // every evaluation function forwards it along. However, in the case of `ReturnValue`s, the
    // `call_function_object` function handles the error, unwraps the value returned, and doesn't
    // forward it further. If there is a return statement outside of a function context, there will
    // be no `call_function_object` call in the call stack, and the error will be forwarded along
    // all the way to the root call. Now, whether this was in the REPL or the code was being
    // executed from a file, the error will be interpreted as an invalid return -- a return
    // statement ouside of a function context -- and will be handled like any other runtime error.
    ReturnValue(Box<Object>),
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use RuntimeError::*;
        match self {
            IdenNotFound(s) => write!(f, "identifier not found: '{}'", s),
            WrongNumberOfArgs(expected, got) => write!(
                f,
                "wrong number of arguments: expected {} arguments but {} were given",
                expected, got
            ),
            IndexTypeError(obj) => write!(f, "index must be integer, not '{}'", obj),
            IndexOutOfBounds(i) => write!(f, "index out of bounds: {}", i),
            HashKeyTypeError(obj) => write!(f, "hash key must be hashable type, not '{}'", obj),
            KeyError(obj) => write!(f, "hash key error: entry for {} not found", obj),
            IndexingWrongType(obj) => write!(f, "'{}' is not an array or hash object", obj),
            PrefixTypeError(tk, obj) => write!(
                f,
                "unsuported operand type for prefix operator {}: '{}'",
                tk, obj
            ),
            InfixTypeError(left, tk, right) => write!(
                f,
                "unsuported operand types for infix operator {}: '{}' and '{}'",
                tk, left, right,
            ),
            NotCallable(obj) => {
                write!(f, "'{}' is not a function object or built-in function", obj)
            }
            DivOrModByZero => write!(f, "division or modulo by zero"),
            NegativeExponent => write!(f, "negative exponent"),
            TypeError(expected, got) => {
                write!(f, "type error: expected '{}', got '{}'", expected, got)
            }
            Custom(msg) => write!(f, "{}", msg),
            
            StackOverflow => write!(f, "stack overflow"),
            StackUnderflow => write!(f, "stack underflow"),

            // A `ReturnValue` that was not handled by `call_function_object` means that it was
            // located outside a function context.
            ReturnValue(_) => write!(f, "`return` outside of function context"),
        }
    }
}
