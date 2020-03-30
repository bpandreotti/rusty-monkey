use crate::ast::Statement;
use crate::builtins::*;
use crate::environment::*;

use std::collections::HashMap;
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
    Array(Vec<Object>),
    Hash(HashMap<HashableObject, Object>),
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
            Object::Array(v) => {
                if v.is_empty() {
                    return write!(f, "[]");
                }

                write!(f, "[{}", v[0])?;
                for element in &v[1..] {
                    write!(f, ", {}", element)?;
                }
                write!(f, "]")
            }
            Object::Hash(h) => {
                if h.is_empty() {
                    return write!(f, "{{}}");
                }

                write!(f, "{{")?;
                let entries: Vec<_> =  h.iter().collect();
                write!(f, "{}: {}", entries[0].0, entries[0].1)?;
                for (key, val) in &entries[1..] {
                    write!(f, ", {}: {}", key, val)?;
                }
                write!(f, "}}")
            }
            Object::Nil => write!(f, "nil"),
            Object::Function(_) => write!(f, "<function>"),
            Object::Builtin(_) => write!(f, "<built-in function>"),
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
            Object::Array(_) => "array",
            Object::Hash(_) => "hash",
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum HashableObject {
    Str(String),
    Integer(i64),
    Boolean(bool),
}

impl fmt::Display for HashableObject {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            HashableObject::Integer(i) => write!(f, "{}", i),
            HashableObject::Boolean(b) => write!(f, "{}", b),
            HashableObject::Str(s) => write!(f, "\"{}\"", s.escape_debug()),
        }
    }
}

impl HashableObject {
    pub fn from_object(obj: Object) -> Option<HashableObject> {
        match obj {
            Object::Str(s) => Some(HashableObject::Str(s)),
            Object::Integer(i) => Some(HashableObject::Integer(i)),
            Object::Boolean(b) => Some(HashableObject::Boolean(b)),
            _ => None,
        }
    }
}
