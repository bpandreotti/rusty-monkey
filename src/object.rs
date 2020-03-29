use crate::ast::Statement;
use crate::builtins::*;
use crate::environment::*;

use std::fmt;

#[derive(Debug, Clone)]
pub struct FunctionObject {
    pub environment: EnvHandle,
    pub parameters: Vec<String>,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub enum Object {
    Integer(i64),
    Boolean(bool),
    Str(String),
    ReturnValue(Box<Object>),
    Function(FunctionObject),
    Builtin(BuiltinFn),
    Nil,
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Object::Integer(i) => write!(f, "{}", i),
            Object::Boolean(b) => write!(f, "{}", b),
            Object::Str(s) => write!(f, "\"{}\"", s.escape_debug()),
            Object::Nil => write!(f, "nil"),
            Object::Function(_) => write!(f, "[function]"),
            Object::Builtin(_) => write!(f, "[built-in function]"),
            Object::ReturnValue(_) => panic!("trying to display ReturnValue object"),
        }
    }
}

impl Object {
    pub fn type_str(&self) -> &'static str {
        match self {
            Object::Integer(_) => "int",
            Object::Boolean(_) => "bool",
            Object::Str(_) => "string",
            Object::Nil => "nil",
            Object::ReturnValue(_) => "ReturnValue object",
            Object::Function(_) => "function",
            Object::Builtin(_) => "function",
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Object::Boolean(false) | Object::Nil | Object::Integer(0) => false,
            _ => true,
        }
    }

    pub fn unwrap_return_value(self) -> Object {
        match self {
            Object::ReturnValue(v) => v.unwrap_return_value(),
            other => other,
        }
    }
}
