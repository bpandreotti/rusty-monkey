use crate::environment::*;
use crate::ast::Statement;

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
    ReturnValue(Box<Object>),
    Function(FunctionObject),
    Nil,
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Object::Integer(i) => write!(f, "{}", i),
            Object::Boolean(b) => write!(f, "{}", b),
            Object::Nil => write!(f, "nil"),
            Object::ReturnValue(v) => write!(f, "return({})", v), // @DEBUG

            // @DEBUG
            Object::Function(fo) => {
                writeln!(f, "fn")?;
                writeln!(f, "{:?}", fo.parameters)?;
                writeln!(f, "{:?}", fo.body)
            }
        }
    }
}

impl Object {
    pub fn type_str(&self) -> &'static str {
        match self {
            Object::Integer(_) => "int",
            Object::Boolean(_) => "bool",
            Object::Nil => "nil",
            Object::ReturnValue(_) => "ReturnValue object",
            Object::Function(_) => "function"
        }
    }
}
