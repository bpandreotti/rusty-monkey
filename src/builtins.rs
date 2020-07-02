use crate::error::*;
use crate::object::*;

use std::fmt;

#[derive(Clone)]
pub struct BuiltinFn(pub fn(Vec<Object>) -> Result<Object, RuntimeError>);

impl fmt::Debug for BuiltinFn {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "BuiltinFn")
    }
}

#[cfg(test)]
impl PartialEq for BuiltinFn {
    fn eq(&self, _: &BuiltinFn) -> bool {
        panic!("Trying to compare `BuiltinFn`s")
    }
}

pub const ALL_BUILTINS: [(&str, BuiltinFn); 9] = [
    ("type", BuiltinFn(builtin_type)),
    ("puts", BuiltinFn(builtin_puts)),
    ("len", BuiltinFn(builtin_len)),
    ("push", BuiltinFn(builtin_push)),
    ("cons", BuiltinFn(builtin_cons)),
    ("head", BuiltinFn(builtin_head)),
    ("tail", BuiltinFn(builtin_tail)),
    ("range", BuiltinFn(builtin_range)),
    ("assert", BuiltinFn(builtin_assert)),
];

pub fn get_builtin(name: &str) -> Option<Object> {
    ALL_BUILTINS
        .iter()
        .find(|(s, _)| s == &name)
        .map(|(_, f)| Object::Builtin(f.clone()))
}

fn assert_num_arguments(args: &[Object], expected: usize) -> Result<(), RuntimeError> {
    if args.len() != expected {
        Err(RuntimeError::WrongNumberOfArgs(expected, args.len()))
    } else {
        Ok(())
    }
}

fn assert_object_type_integer(obj: &Object) -> Result<&i64, RuntimeError> {
    if let Object::Integer(i) = obj {
        Ok(i)
    } else {
        Err(RuntimeError::TypeError(
            Object::Integer(0).type_str(),
            obj.type_str(),
        ))
    }
}

fn assert_object_type_array(obj: &Object) -> Result<&Vec<Object>, RuntimeError> {
    if let Object::Array(a) = obj {
        Ok(a)
    } else {
        Err(RuntimeError::TypeError(
            Object::Array(Box::new(vec![])).type_str(),
            obj.type_str(),
        ))
    }
}

fn builtin_type(args: Vec<Object>) -> Result<Object, RuntimeError> {
    assert_num_arguments(&args, 1)?;
    Ok(Object::from(args[0].type_str()))
}

fn builtin_puts(args: Vec<Object>) -> Result<Object, RuntimeError> {
    if args.is_empty() {
        return Err(RuntimeError::WrongNumberOfArgs(1, 0));
    }

    for arg in &args[..args.len() - 1] {
        print!("{} ", arg);
    }
    println!("{}", args[args.len() - 1]);

    Ok(Object::Nil)
}

fn builtin_len(args: Vec<Object>) -> Result<Object, RuntimeError> {
    assert_num_arguments(&args, 1)?;

    let length = match &args[0] {
        Object::Str(s) => s.chars().count(),
        Object::Array(a) => a.len(),
        o => {
            return Err(RuntimeError::Custom(format!(
                "'{}' object has no length",
                o.type_str()
            )))
        }
    };

    Ok(Object::Integer(length as i64))
}

fn builtin_push(args: Vec<Object>) -> Result<Object, RuntimeError> {
    assert_num_arguments(&args, 2)?;
    let mut array = assert_object_type_array(&args[0])?.clone();
    array.push(args[1].clone());
    Ok(Object::Array(Box::new(array)))
}

fn builtin_cons(args: Vec<Object>) -> Result<Object, RuntimeError> {
    assert_num_arguments(&args, 2)?;
    let tail = assert_object_type_array(&args[1])?;
    let mut new = vec![args[0].clone()];
    new.extend_from_slice(tail);
    Ok(Object::Array(Box::new(new)))
}

fn builtin_head(args: Vec<Object>) -> Result<Object, RuntimeError> {
    assert_num_arguments(&args, 1)?;
    let array = assert_object_type_array(&args[0])?;
    if let Some(obj) = array.get(0) {
        Ok(obj.clone())
    } else {
        Ok(Object::Nil)
    }
}

fn builtin_tail(args: Vec<Object>) -> Result<Object, RuntimeError> {
    assert_num_arguments(&args, 1)?;
    let array = assert_object_type_array(&args[0])?;
    match array.get(1..) {
        Some(tail) => Ok(Object::Array(Box::new(tail.to_vec()))),
        None => Ok(Object::Nil),
    }
}

fn builtin_range(args: Vec<Object>) -> Result<Object, RuntimeError> {
    if args.is_empty() {
        return Err(RuntimeError::WrongNumberOfArgs(1, 0));
    } else if args.len() > 3 {
        return Err(RuntimeError::WrongNumberOfArgs(3, args.len()));
    }

    let mut end = *assert_object_type_integer(&args[0])?;

    let mut start = 0;
    if args.len() >= 2 {
        start = end;
        end = *assert_object_type_integer(&args[1])?;
    }

    let step = if args.len() >= 3 {
        *assert_object_type_integer(&args[2])?
    } else {
        1
    };

    if step <= 0 {
        return Err(RuntimeError::Custom(
            "Third argument to `range` must be positive".into(),
        ));
    }

    Ok(Object::Array(Box::new(
        (start..end)
            .step_by(step as usize)
            .map(Object::Integer)
            .collect(),
    )))
}

fn builtin_assert(args: Vec<Object>) -> Result<Object, RuntimeError> {
    assert_num_arguments(&args, 1)?;
    if args[0].is_truthy() {
        Ok(Object::Nil)
    } else {
        Err(RuntimeError::Custom(format!(
            "Assertion failed on value {}",
            args[0]
        )))
    }
}
