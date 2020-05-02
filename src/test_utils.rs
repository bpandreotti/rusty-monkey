use crate::compiler::{self, code};
use crate::error::*;
use crate::interpreter::object;
use crate::parser;

pub fn parse_and_compile(program: &str) -> Result<code::Bytecode, MonkeyError> {
    let parsed = parser::parse(program.into())?;
    let mut comp = compiler::Compiler::new();
    comp.compile_block(parsed)?;
    Ok(comp.bytecode())
}

pub fn compare_objects(left: &object::Object, right: &object::Object) -> bool {
    use object::Object::*;

    // Nil, integer, boolean and string comparisons are done directly. Array comparison is done by
    // recursively comparing each element. Hash comparison is done by formatting the hashes into
    // strings, using the `Display` implementation for object. Function and built-in comparisons are
    // unsupported, and always return false.
    match (left, right) {
        (Nil, Nil) => true,
        (Integer(x), Integer(y)) => x == y,
        (Boolean(p), Boolean(q)) => p == q,
        (Str(r), Str(s)) => r == s,
        (Array(a), Array(b)) => {
            a.len() == b.len() && a.iter().zip(b).all(|(l, r)| compare_objects(l, r))
        }
        (Hash(_), Hash(_)) => format!("{}", left) == format!("{}", right),
        _ => false,
    }
}
