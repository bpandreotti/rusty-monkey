// @WIP
use crate::builtins::BuiltinFn;
use crate::compiler::code;
use crate::interpreter::environment;
use crate::parser::ast;
use std::collections::HashMap;
use std::convert::From;
use std::fmt;

#[derive(Debug, Clone)]
#[cfg_attr(test, derive(PartialEq))]
pub enum Object {
    Nil,
    Integer(i64),
    Boolean(bool),
    // @PERFORMANCE: Since strings are immutable in monkey, it might be better to use a `Box<str>`.
    Str(Box<String>),
    // @PERFORMANCE: We use `Box<Vec<_>>` instead of just `Vec<_>` because we want the object
    // representation to be as small as possible. Currently the size of `Object` is 16 bytes -- if
    // we used just `Vec<_>` it would be 32.
    #[allow(clippy::box_vec)]
    Array(Box<Vec<Object>>),
    Hash(Box<HashMap<HashableObject, Object>>),
    CompiledFunc(Box<CompiledFunction>),
    Closure(Box<Closure>),
    InterpreterFunc(Box<InterpreterFunctionObject>),
    Builtin(BuiltinFn),
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Object::Nil => write!(f, "nil"),
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
            Object::CompiledFunc(_) | Object::Closure(_) | Object::InterpreterFunc(_) => {
                write!(f, "<function>")
            }
            Object::Builtin(_) => write!(f, "<built-in function>"),
        }
    }
}

impl Object {
    pub fn type_str(&self) -> &'static str {
        use Object::*;
        match self {
            Nil => "nil",
            Integer(_) => "int",
            Boolean(_) => "bool",
            Str(_) => "string",
            Array(_) => "array",
            Hash(_) => "hash",
            CompiledFunc(_) | Closure(_) | InterpreterFunc(_) | Builtin(_) => "function",
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Object::Boolean(false) | Object::Nil | Object::Integer(0) => false,
            _ => true,
        }
    }

    pub fn eq(left: &Object, right: &Object) -> Option<bool> {
        // Function, array, and hash comparisons are unsupported
        match (left, right) {
            (Object::Nil, Object::Nil) => Some(true),
            (Object::Integer(l), Object::Integer(r)) => Some(l == r),
            (Object::Boolean(l), Object::Boolean(r)) => Some(l == r),
            (Object::Str(l), Object::Str(r)) => Some(l == r),
            _ => None,
        }
    }
}

impl From<&str> for Object {
    fn from(s: &str) -> Self {
        Object::Str(Box::new(s.into()))
    }    
}

#[derive(Debug, Clone)]
#[cfg_attr(test, derive(PartialEq))]
pub struct CompiledFunction {
    pub instructions: code::Instructions,
    pub num_locals: u8,
    pub num_params: u8,
}

#[derive(Debug, Clone)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Closure {
    pub func: CompiledFunction,
    pub free_vars: Vec<Object>,
}

#[derive(Debug, Clone)]
pub struct InterpreterFunctionObject {
    pub environment: environment::EnvHandle,
    pub parameters: Vec<String>,
    pub body: Vec<ast::NodeStatement>,
}

#[cfg(test)]
impl PartialEq for InterpreterFunctionObject {
    fn eq(&self, _: &Self) -> bool {
        panic!("Trying to compare `InterpreterFunctionObject`s")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HashableObject {
    Nil,
    Integer(i64),
    Boolean(bool),
    Str(Box<String>),
}

impl fmt::Display for HashableObject {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            HashableObject::Nil => write!(f, "nil"),
            HashableObject::Integer(i) => write!(f, "{}", i),
            HashableObject::Boolean(b) => write!(f, "{}", b),
            HashableObject::Str(s) => write!(f, "\"{}\"", s.escape_debug()),
        }
    }
}

impl HashableObject {
    pub fn from_object(obj: Object) -> Option<HashableObject> {
        match obj {
            Object::Nil => Some(HashableObject::Nil),
            Object::Integer(i) => Some(HashableObject::Integer(i)),
            Object::Boolean(b) => Some(HashableObject::Boolean(b)),
            Object::Str(s) => Some(HashableObject::Str(s)),
            _ => None,
        }
    }
}

impl From<&str> for HashableObject {
    fn from(s: &str) -> Self {
        HashableObject::Str(Box::new(s.into()))
    }    
}
