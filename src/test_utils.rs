use crate::parser;
use crate::interpreter::object;
use crate::compiler::{self, code};
use crate::error::*;

pub fn parse_and_compile(program: &str) -> Result<code::Bytecode, MonkeyError> {
    let parsed = parser::parse(program.into())?;
    let mut comp = compiler::Compiler::new();
    comp.compile_block(parsed)?;
    Ok(comp.bytecode())
}

pub fn compare_objects(left: &object::Object, right: &object::Object) -> bool {
    // Since `object::Object`s can't be compared directly, we format them to strings and compare
    // those. This works on the assumption that two equal objects have the same `Debug`
    // representation and vice-versa.
    format!("{:?}", left) == format!("{:?}", right)
}
