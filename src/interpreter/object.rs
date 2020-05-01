use crate::parser::ast::NodeStatement;
use super::builtins::BuiltinFn;
use super::environment::EnvHandle;

use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone)]
pub struct FunctionObject {
    pub environment: EnvHandle,
    pub parameters: Vec<String>,
    pub body: Vec<NodeStatement>,
}

#[derive(Debug, Clone)]
pub enum Object {
    Integer(i64),
    Boolean(bool),
    Str(String),
    Array(Vec<Object>),
    Hash(HashMap<HashableObject, Object>),
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
                    return write!(f, "#{{}}");
                }

                write!(f, "#{{")?;
                let mut keys_sorted = h.keys().collect::<Vec<_>>();
                keys_sorted.sort(); // We sort the keys so the hash printing will be consistent
                write!(f, "{}: {}", keys_sorted[0], h[keys_sorted[0]])?;
                for key in &keys_sorted[1..] {
                    write!(f, ", {}: {}", key, h[key])?;
                }
                write!(f, "}}")
            }
            Object::Nil => write!(f, "nil"),
            Object::Function(_) => write!(f, "<function>"),
            Object::Builtin(_) => write!(f, "<built-in function>"),
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

    pub fn are_equal(left: &Object, right: &Object) -> Option<bool> {
        // Function object, array, and hash comparisons are unsupported
        match (left, right) {
            (Object::Integer(l), Object::Integer(r)) => Some(l == r),
            (Object::Boolean(l), Object::Boolean(r)) => Some(l == r),
            (Object::Str(l), Object::Str(r)) => Some(l == r),
            (Object::Nil, Object::Nil) => Some(true),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
