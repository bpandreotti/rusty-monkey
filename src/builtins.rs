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
        "map" => make_builtin!(builtin_map),
        "range" => make_builtin!(builtin_range),
        "assert" => make_builtin!(builtin_assert),
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

fn assert_object_type_integer(obj: &Object) -> Result<&i64, RuntimeError> {
    if let Object::Integer(i) = obj {
        Ok(i)
    } else {
        Err(RuntimeError::TypeError(Object::Integer(0).type_str(), obj.type_str()))
    }
}

fn assert_object_type_array(obj: &Object) -> Result<&Vec<Object>, RuntimeError> {
    if let Object::Array(a) = obj {
        Ok(a)
    } else {
        Err(RuntimeError::TypeError(Object::Array(vec![]).type_str(), obj.type_str()))
    }
}

fn assert_object_type_string(obj: &Object) -> Result<&String, RuntimeError> {
    if let Object::Str(s) = obj {
        Ok(s)
    } else {
        Err(RuntimeError::TypeError(Object::Str("".into()).type_str(), obj.type_str()))
    }
}

fn assert_object_type_function(obj: &Object) -> Result<&FunctionObject, RuntimeError> {
    if let Object::Function(fo) = obj {
        Ok(fo)
    } else {
        Err(RuntimeError::TypeError("function", obj.type_str()))
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
    let mut array = assert_object_type_array(&args[0])?.clone();
    array.push(args[1].clone());
    Ok(Object::Array(array))
}

fn builtin_cons(args: Vec<Object>, _: &EnvHandle) -> Result<Object, RuntimeError> {
    assert_num_arguments(&args, 2)?;
    let tail = assert_object_type_array(&args[1])?;
    let mut new = vec![args[0].clone()];
    new.extend_from_slice(tail);
    Ok(Object::Array(new))
}

fn builtin_head(args: Vec<Object>, _: &EnvHandle) -> Result<Object, RuntimeError> {
    assert_num_arguments(&args, 1)?;
    let array = assert_object_type_array(&args[0])?;
    if let Some(obj) = array.get(0) {
        Ok(obj.clone())
    } else {
        Ok(Object::Nil)
    }
}

fn builtin_tail(args: Vec<Object>, _: &EnvHandle) -> Result<Object, RuntimeError> {
    assert_num_arguments(&args, 1)?;
    let array = assert_object_type_array(&args[0])?;
    match array.get(1..) {
        Some(tail) => Ok(Object::Array(tail.to_vec())),
        None => Ok(Object::Nil),
    }
}

fn builtin_import(args: Vec<Object>, env: &EnvHandle) -> Result<Object, RuntimeError> {
    use crate::lexer::Lexer;
    use crate::parser::Parser;
    use std::fs;

    assert_num_arguments(&args, 1)?;
    let file_name = assert_object_type_string(&args[0])?;
    // @TODO: Read file using BufRead instead of reading to string
    let contents = fs::read_to_string(file_name)
        .map_err(|e| RuntimeError::Custom(format!("File error: {}", e)))?;
    let lexer = Lexer::from_string(contents)
        .map_err(|e| RuntimeError::Custom(format!("Error constructing lexer: {}", e)))?;
    let parsed_program = Parser::new(lexer)
        .map_err(|e| RuntimeError::Custom(format!("Error constructing parser: {}", e)))?
        .parse_program()
        .map_err(|e| RuntimeError::Custom(format!("Parser error: {}", e)))?;
    for statement in parsed_program {
        eval::eval_statement(&statement, &env).map_err(|e| {
            RuntimeError::Custom(format!("Error while evaluating imported file: {}", e))
        })?;
    }
    Ok(Object::Nil)
}

// @TODO: Add support for mapping built-ins. Maybe merge the object representation of
// FunctionObject and BuiltinFn into a "Callable" enum?
fn builtin_map(args: Vec<Object>, _: &EnvHandle) -> Result<Object, RuntimeError> {
    assert_num_arguments(&args, 2)?;
    let fo = assert_object_type_function(&args[0])?;
    let array = assert_object_type_array(&args[1])?;

    let mut new_vector = Vec::new();
    for element in array {
        let call_result = eval::call_function_object(fo.clone(), vec![element.clone()], (0, 0));
        match call_result {
            Ok(v) => new_vector.push(v),
            Err(monkey_err) => match monkey_err.error {
                ErrorType::Runtime(e) => return Err(e),
                _ => unreachable!(),
            }
        }
    }
    Ok(Object::Array(new_vector))
}

fn builtin_range(args: Vec<Object>, _: &EnvHandle) -> Result<Object, RuntimeError> {
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
        return Err(RuntimeError::Custom("Third argument to `range` must be positive".into()))
    }

    Ok(Object::Array(
        (start..end).step_by(step as usize).map(|i| { Object::Integer(i) }).collect()
    ))
}

fn builtin_assert(args: Vec<Object>, _: &EnvHandle) -> Result<Object, RuntimeError> {
    assert_num_arguments(&args, 1)?;
    if args[0].is_truthy() {
        Ok(Object::Nil)
    } else {
        Err(RuntimeError::Custom(format!("Assertion failed on value {}", args[0])))
    }
}
