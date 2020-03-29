use crate::eval::*;
use crate::object::*;

pub type BuiltinFn = fn(Vec<Object>) -> EvalResult;

pub fn get_builtin(name: &str) -> Option<Object> {
    match name {
        "type" => Some(Object::Builtin(builtin_type)),
        "len" => Some(Object::Builtin(builtin_len)),
        "puts" => Some(Object::Builtin(builtin_puts)),
        "push" => Some(Object::Builtin(builtin_push)),
        "cons" => Some(Object::Builtin(builtin_cons)),
        "hd" => Some(Object::Builtin(builtin_hd)),
        "tl" => Some(Object::Builtin(builtin_tl)),
        _ => None,
    }
}

fn assert_num_arguments(args: &[Object], expected: usize) -> Result<(), RuntimeError> {
    if args.len() != expected {
        crate::runtime_err!(
            "Wrong number of arguments. Expected {} arguments, {} were given",
            expected,
            args.len()
        )
    } else {
        Ok(())
    }
}

fn builtin_type(args: Vec<Object>) -> EvalResult {
    assert_num_arguments(&args, 1)?;
    Ok(Object::Str(args[0].type_str().into()))
}

fn builtin_len(args: Vec<Object>) -> EvalResult {
    assert_num_arguments(&args, 1)?;

    let length = match &args[0] {
        Object::Str(s) => s.chars().count(),
        Object::Array(a) => a.len(),
        o => return crate::runtime_err!("'{}' object has no `len`", o.type_str()),
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

fn builtin_push(args: Vec<Object>) -> EvalResult {
    assert_num_arguments(&args, 2)?;

    match &args[0] {
        Object::Array(a) => {
            let mut new = a.clone();
            new.push(args[1].clone());
            Ok(Object::Array(new))
        }
        other => crate::runtime_err!(
            "First argument to `push` must be array, got '{}'",
            other.type_str()
        ),
    }
}

fn builtin_cons(args: Vec<Object>) -> EvalResult {
    assert_num_arguments(&args, 2)?;

    match &args[1] {
        Object::Array(a) => {
            let mut new = vec![args[0].clone()];
            new.extend_from_slice(&a);
            Ok(Object::Array(new))
        }
        other => crate::runtime_err!(
            "Second argument to `cons` must be array, got '{}'",
            other.type_str()
        ),
    }
}

fn builtin_hd(args: Vec<Object>) -> EvalResult {
    assert_num_arguments(&args, 1)?;

    match &args[0] {
        Object::Array(a) => {
            if let Some(obj) = a.get(0) {
                Ok(obj.clone())
            } else {
                Ok(Object::Nil)
            }
        }
        other => crate::runtime_err!("Argument to `hd` must be array, got '{}'", other.type_str()),
    }
}

fn builtin_tl(args: Vec<Object>) -> EvalResult {
    assert_num_arguments(&args, 1)?;

    match &args[0] {
        Object::Array(a) => match a.get(1..) {
            Some(tail) => Ok(Object::Array(tail.to_vec())),
            None => Ok(Object::Nil),
        },
        other => crate::runtime_err!("Argument to `tl` must be array, got '{}'", other.type_str()),
    }
}
