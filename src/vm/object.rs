use crate::compiler::code;
use crate::hashable::HashableObject;

use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone)]
pub enum Object {
    Integer(i64),
    Boolean(bool),
    Str(String),
    Array(Vec<Object>),
    Hash(HashMap<HashableObject, Object>),
    Nil,
    // @TODO: Extract this into a separate struct
    CompiledFunction {
        instructions: code::Instructions,
        num_locals: u8,
        num_params: u8,
    }
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
            Object::CompiledFunction { .. } => write!(f, "<function>"),
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
            Object::CompiledFunction { .. } => "function",
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Object::Boolean(false) | Object::Nil | Object::Integer(0) => false,
            _ => true,
        }
    }
}
