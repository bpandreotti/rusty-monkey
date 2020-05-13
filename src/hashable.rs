use crate::vm::object as vm;

use std::fmt;

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
    pub fn from_vm_object(obj: vm::Object) -> Option<HashableObject> {
        match obj {
            vm::Object::Str(s) => Some(HashableObject::Str(s)),
            vm::Object::Integer(i) => Some(HashableObject::Integer(i)),
            vm::Object::Boolean(b) => Some(HashableObject::Boolean(b)),
            _ => None,
        }
    }
}
