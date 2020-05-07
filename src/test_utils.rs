use crate::compiler::{self, code};
use crate::error::*;
use crate::interpreter;
use crate::parser;
use crate::vm;

pub fn parse_and_compile(program: &str) -> Result<code::Bytecode, MonkeyError> {
    let parsed = parser::parse(program.into())?;
    let mut comp = compiler::Compiler::new();
    comp.compile_block(parsed)?;
    Ok(comp.bytecode())
}

pub fn compare_interpreter_objects(
    left: &interpreter::object::Object,
    right: &interpreter::object::Object,
) -> bool {
    use interpreter::object::Object::*;

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
            a.len() == b.len()
                && a.iter()
                    .zip(b)
                    .all(|(l, r)| compare_interpreter_objects(l, r))
        }
        (Hash(_), Hash(_)) => format!("{}", left) == format!("{}", right),
        _ => false,
    }
}

pub fn compare_vm_objects(left: &vm::object::Object, right: &vm::object::Object) -> bool {
    use vm::object::Object::*;
    // Same thing as the previous function, but for VM objects
    match (left, right) {
        (Nil, Nil) => true,
        (Integer(x), Integer(y)) => x == y,
        (Boolean(p), Boolean(q)) => p == q,
        (Str(r), Str(s)) => r == s,
        (Array(a), Array(b)) => {
            a.len() == b.len() && a.iter().zip(b).all(|(l, r)| compare_vm_objects(l, r))
        }
        (Hash(_), Hash(_)) => format!("{}", left) == format!("{}", right),
        (CompiledFunction(f), CompiledFunction(g)) => f.0 == g.0,
        _ => false,
    }
}
