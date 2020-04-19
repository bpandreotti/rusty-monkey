use crate::environment::*;
use crate::error::*;
use crate::eval;
use crate::object::*;

use std::fmt;

#[derive(Clone)]
pub struct BuiltinFn(pub fn(Vec<Object>, env: &EnvHandle) -> Result<Object, RuntimeError>);

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
        "head" => make_builtin!(builtin_head),
        "tail" => make_builtin!(builtin_tail),
        "import" => make_builtin!(builtin_import),
        _ => None,
    }
}

fn assert_num_arguments(args: &[Object], expected: usize) -> Result<(), RuntimeError> {
    if args.len() != expected {
        Err(RuntimeError::WrongNumberOfArgs(expected, args.len()))
    } else {
        Ok(())
    }
}

fn builtin_type(args: Vec<Object>, _: &EnvHandle) -> Result<Object, RuntimeError> {
    assert_num_arguments(&args, 1)?;
    Ok(Object::Str(args[0].type_str().into()))
}

fn builtin_puts(args: Vec<Object>, _: &EnvHandle) -> Result<Object, RuntimeError> {
    if args.is_empty() {
        return Err(RuntimeError::WrongNumberOfArgs(1, 0));
    }

    for arg in &args[..args.len() - 1] {
        print!("{} ", arg);
    }
    println!("{}", args[args.len() - 1]);

    Ok(Object::Nil)
}

fn builtin_len(args: Vec<Object>, _: &EnvHandle) -> Result<Object, RuntimeError> {
    assert_num_arguments(&args, 1)?;

    let length = match &args[0] {
        Object::Str(s) => s.chars().count(),
        Object::Array(a) => a.len(),
        o => return Err(RuntimeError::Custom(
            format!("'{}' object has no length", o.type_str())
        )),
    };

    Ok(Object::Integer(length as i64))
}

fn builtin_get(args: Vec<Object>, _: &EnvHandle) -> Result<Object, RuntimeError> {
    assert_num_arguments(&args, 2)?;

    // Evaluate an index expression using the arguments passed and deal with any error encountered
    eval::eval_index_expression(&args[0], &args[1]).or_else(|error| match error {
        // If the error is IndexOutOfBounds or KeyError, we return 'nil'
        RuntimeError::IndexOutOfBounds(_) | RuntimeError::KeyError(_) =>  Ok(Object::Nil),
        // Otherwise, we forward the error
        _ => Err(error),
    })
}

fn builtin_push(args: Vec<Object>, _: &EnvHandle) -> Result<Object, RuntimeError> {
    assert_num_arguments(&args, 2)?;

    match &args[0] {
        Object::Array(a) => {
            let mut new = a.clone();
            new.push(args[1].clone());
            Ok(Object::Array(new))
        }
        other => Err(RuntimeError::Custom(
            format!("First argument to `push` must be array, got '{}'", other.type_str())
        )),
    }
}

fn builtin_cons(args: Vec<Object>, _: &EnvHandle) -> Result<Object, RuntimeError> {
    assert_num_arguments(&args, 2)?;

    match &args[1] {
        Object::Array(a) => {
            let mut new = vec![args[0].clone()];
            new.extend_from_slice(&a);
            Ok(Object::Array(new))
        }
        other => Err(RuntimeError::Custom(
            format!("Second argument to `cons` must be array, got '{}'", other.type_str())
        )),
    }
}

fn builtin_head(args: Vec<Object>, _: &EnvHandle) -> Result<Object, RuntimeError> {
    assert_num_arguments(&args, 1)?;

    match &args[0] {
        Object::Array(a) => {
            if let Some(obj) = a.get(0) {
                Ok(obj.clone())
            } else {
                Ok(Object::Nil)
            }
        }
        other => Err(RuntimeError::Custom(
            format!("Argument to `hd` must be array, got '{}'", other.type_str())
        )),
    }
}

fn builtin_tail(args: Vec<Object>, _: &EnvHandle) -> Result<Object, RuntimeError> {
    assert_num_arguments(&args, 1)?;

    match &args[0] {
        Object::Array(a) => match a.get(1..) {
            Some(tail) => Ok(Object::Array(tail.to_vec())),
            None => Ok(Object::Nil),
        },
        other => Err(RuntimeError::Custom(
            format!("Argument to `tl` must be array, got '{}'", other.type_str())
        )),
    }
}

fn builtin_import(args: Vec<Object>, env: &EnvHandle) -> Result<Object, RuntimeError> {
    use crate::lexer::Lexer;
    use crate::parser::Parser;
    use std::fs;

    assert_num_arguments(&args, 1)?;

    if let Object::Str(file_name) = &args[0] {
        let contents = fs::read_to_string(file_name)
            .map_err(|e| RuntimeError::Custom(format!("File error: {}", e)))?;
        let lexer = Lexer::from_string(contents);
        let parsed_program = Parser::new(lexer)
            .parse_program()
            .map_err(|e| RuntimeError::Custom(format!("Parser error: {}", e)))?;
        for statement in parsed_program {
            eval::eval_statement(&statement, &env).map_err(|e| {
                RuntimeError::Custom(format!("Error while evaluating imported file: {}", e))
            })?;
        }
        Ok(Object::Nil)
    } else {
        Err(RuntimeError::Custom(
            format!("Argument to `import` must be string, got '{}'", args[0].type_str())
        ))
    }
}
