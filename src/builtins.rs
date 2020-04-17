use crate::environment::*;
use crate::eval::*;
use crate::object::*;

use std::fmt;

#[derive(Clone)]
pub struct BuiltinFn(pub fn(Vec<Object>, env: &EnvHandle) -> EvalResult);

impl fmt::Debug for BuiltinFn {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "BuiltinFn")
    }
}

macro_rules! make_builtin {
    ($x:expr) => {
        Some(Object::Builtin(BuiltinFn($x)))
    };
}

pub fn get_builtin(name: &str) -> Option<Object> {
    match name {
        "type" => make_builtin!(builtin_type),
        "puts" => make_builtin!(builtin_puts),
        "len" => make_builtin!(builtin_len),
        "get" => make_builtin!(builtin_get),
        "push" => make_builtin!(builtin_push),
        "cons" => make_builtin!(builtin_cons),
        "hd" => make_builtin!(builtin_hd),
        "tl" => make_builtin!(builtin_tl),
        "import" => make_builtin!(builtin_import),
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

fn builtin_type(args: Vec<Object>, _: &EnvHandle) -> EvalResult {
    assert_num_arguments(&args, 1)?;
    Ok(Object::Str(args[0].type_str().into()))
}

fn builtin_puts(args: Vec<Object>, _: &EnvHandle) -> EvalResult {
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

fn builtin_len(args: Vec<Object>, _: &EnvHandle) -> EvalResult {
    assert_num_arguments(&args, 1)?;

    let length = match &args[0] {
        Object::Str(s) => s.chars().count(),
        Object::Array(a) => a.len(),
        o => return crate::runtime_err!("'{}' object has no `len`", o.type_str()),
    };

    Ok(Object::Integer(length as i64))
}

fn builtin_get(args: Vec<Object>, _: &EnvHandle) -> EvalResult {
    // @TODO: maybe call `eval::eval_index_expression` and handle the errors instead?
    assert_num_arguments(&args, 2)?;

    let object = match &args[0] {
        Object::Array(a) => {
            if let Object::Integer(i) = args[1] {
                if i < 0 {
                    None
                } else {
                    a.get(i as usize)
                }
            } else {
                return crate::runtime_err!(
                    "Array index must be integer, not '{}'",
                    args[1].type_str()
                );
            }
        }
        Object::Hash(h) => {
            let key_type = args[1].type_str();
            match HashableObject::from_object(args[1].clone()) {
                Some(k) => h.get(&k),
                None => {
                    return crate::runtime_err!(
                        "Hash key must be hashable type, not '{}'",
                        key_type
                    )
                }
            }
        }
        o => return crate::runtime_err!("'{}' is not an array or hash object", o.type_str()),
    };
    Ok(object.cloned().or(Some(Object::Nil)).unwrap())
}

fn builtin_push(args: Vec<Object>, _: &EnvHandle) -> EvalResult {
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

fn builtin_cons(args: Vec<Object>, _: &EnvHandle) -> EvalResult {
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

fn builtin_hd(args: Vec<Object>, _: &EnvHandle) -> EvalResult {
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

fn builtin_tl(args: Vec<Object>, _: &EnvHandle) -> EvalResult {
    assert_num_arguments(&args, 1)?;

    match &args[0] {
        Object::Array(a) => match a.get(1..) {
            Some(tail) => Ok(Object::Array(tail.to_vec())),
            None => Ok(Object::Nil),
        },
        other => crate::runtime_err!("Argument to `tl` must be array, got '{}'", other.type_str()),
    }
}

fn builtin_import(args: Vec<Object>, env: &EnvHandle) -> EvalResult {
    use crate::lexer::Lexer;
    use crate::parser::Parser;
    use std::fs;

    assert_num_arguments(&args, 1)?;

    if let Object::Str(file_name) = &args[0] {
        let contents = fs::read_to_string(file_name)
            .map_err(|e| RuntimeError(format!("File error: {}", e)))?;
        let lexer = Lexer::from_string(contents);
        let parsed_program = Parser::new(lexer)
            .parse_program()
            .map_err(|e| RuntimeError(format!("Parser error: {}", e)))?;
        for statement in parsed_program {
            eval_statement(&statement, &env)?;
        }
        Ok(Object::Nil)
    } else {
        crate::runtime_err!(
            "Argument to `import` must be string, got '{}'",
            args[0].type_str()
        )
    }
}
