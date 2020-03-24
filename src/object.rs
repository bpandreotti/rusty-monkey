use std::fmt;

#[derive(Debug, PartialEq)]
pub enum Object {
    Integer(i64),
    Boolean(bool),
    Nil, // @TODO: Consider implementing Nil/Null type
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Object::Integer(i) => write!(f, "{}", i),
            Object::Boolean(b) => write!(f, "{}", b),
            Object::Nil => write!(f, "nil"),
        }        
    }
}
