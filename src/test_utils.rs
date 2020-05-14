use crate::compiler::{self, code};
use crate::error::*;
use crate::parser;

macro_rules! monkey_hash {
    ($($key:expr => $value:expr),* $(,)?) => {
        {
            let mut _map = HashMap::new();
            $(_map.insert($key, $value);)*
            crate::object::Object::Hash(Box::new(_map))
        }
    };
}

macro_rules! monkey_array {
    ($($element:expr),* $(,)?) => {
        {
            crate::object::Object::Array(Box::new(
                vec![ $($element),* ]
            ))
        }
    };
}

pub fn parse_and_compile(program: &str) -> Result<code::Bytecode, MonkeyError> {
    let parsed = parser::parse(program.into())?;
    let mut comp = compiler::Compiler::new();
    comp.compile_block(parsed)?;
    Ok(comp.bytecode())
}
