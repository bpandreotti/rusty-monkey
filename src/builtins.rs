use crate::eval::*;
use crate::object::*;

pub type BuiltinFn = fn(Vec<Object>) -> EvalResult;

pub fn get_builtin(name: &str) -> Option<Object> {
    match name {
        "len" => Some(Object::Builtin(builtin_len)),
        "puts" => Some(Object::Builtin(builtin_puts)),
        _ => None,
    }
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

    for arg in &args[..args.len() - 1] {
        print!("{} ", arg);
    }
    println!("{}", args[args.len() - 1]);
    
    Ok(Object::Nil)
}
