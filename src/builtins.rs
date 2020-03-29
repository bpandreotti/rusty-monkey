use crate::eval::*;
use crate::object::*;

type BuiltinFn = fn(Vec<Object>) -> EvalResult;

pub fn builtins() -> Vec<(&'static str, BuiltinFn)> {
    vec![
        ("len", builtin_len),
        ("puts", builtin_puts),
    ]
}

fn builtin_len(args: Vec<Object>) -> EvalResult {
    if args.len() != 1 {
        return crate::runtime_err!(
            "Wrong number of arguments. Expected 1 arguments, {} were given",
            args.len()
        );
    }

    let length = match &args[0] {
        Object::Str(s) => s.chars().count(),
        o => return crate::runtime_err!("'{}' object has no len()", o.type_str()),
    };

    Ok(Object::Integer(length as i64))
}

fn builtin_puts(args: Vec<Object>) -> EvalResult {
    if args.is_empty() {
        return crate::runtime_err!(
            "Wrong number of arguments. Expected 1 or more arguments, 0 were given"
        );
    }

    println!("{}", args[0]);
    for arg in &args[1..] {
        // Print remaining arguments preceded by space
        println!(" {}", arg);
    }
    Ok(Object::Nil)
}
