// @TODO: Document this module
pub mod environment;
#[cfg(test)]
mod tests;

use crate::error::*;
use crate::lexer::token::Token;
use crate::object::*;
use crate::parser::ast::*;
use environment::*;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use RuntimeError::*;

pub fn run_program(program: Vec<NodeStatement>) -> MonkeyResult<()> {
    let env = Rc::new(RefCell::new(Environment::empty()));
    for statement in program {
        eval_statement(&statement, &env)?;
    }
    Ok(())
}

pub fn eval_expression(expression: &NodeExpression, env: &EnvHandle) -> MonkeyResult<Object> {
    match &expression.expression {
        Expression::Identifier(s) => {
            // Note: This clones the object
            match env.borrow().get(&s) {
                Some(value) => Ok(value),
                None => Err(MonkeyError::Interpreter(
                    expression.position,
                    IdenNotFound(s.clone()),
                )),
            }
        }
        Expression::IntLiteral(i) => Ok(Object::Integer(*i)),
        Expression::Boolean(b) => Ok(Object::Boolean(*b)),
        Expression::StringLiteral(s) => Ok(Object::Str(Box::new(s.clone()))),
        Expression::ArrayLiteral(v) => {
            let mut elements = Vec::with_capacity(v.len());
            for exp in v {
                elements.push(eval_expression(exp, env)?);
            }
            Ok(Object::Array(Box::new(elements)))
        }
        Expression::HashLiteral(v) => {
            let mut map = HashMap::new();
            for (key, val) in v {
                let obj = eval_expression(key, env)?;
                let obj_type = obj.type_str();
                let key = match HashableObject::from_object(obj) {
                    Some(v) => v,
                    None => {
                        return Err(MonkeyError::Interpreter(
                            expression.position,
                            HashKeyTypeError(obj_type),
                        ))
                    }
                };

                let val = eval_expression(val, env)?;
                map.insert(key, val);
            }
            Ok(Object::Hash(Box::new(map)))
        }
        Expression::PrefixExpression(tk, e) => {
            let right_side = eval_expression(e, env)?;
            eval_prefix_expression(tk, &right_side)
                .map_err(|e| MonkeyError::Interpreter(expression.position, e))
        }
        Expression::InfixExpression(l, tk, r) => {
            let left_side = eval_expression(l, env)?;
            let right_side = eval_expression(r, env)?;
            eval_infix_expression(&left_side, tk, &right_side)
                .map_err(|e| MonkeyError::Interpreter(expression.position, e))
        }
        Expression::IfExpression {
            condition,
            consequence,
            alternative,
        } => {
            let value = eval_expression(condition, env)?;
            if value.is_truthy() {
                eval_block(consequence, env)
            } else {
                eval_block(alternative, env)
            }
        }
        Expression::Nil => Ok(Object::Nil),
        Expression::FunctionLiteral { parameters, body } => {
            let fo = InterpreterFunctionObject {
                environment: Rc::clone(env),
                parameters: parameters.clone(),
                body: body.clone(),
            };
            Ok(Object::InterpreterFunc(Box::new(fo)))
        }
        Expression::CallExpression {
            function,
            arguments,
        } => {
            // Evaluate the called object
            let obj = eval_expression(function, env)?;
            // Evaluate all arguments sequentially
            let mut evaluated_args = Vec::with_capacity(arguments.len());
            for exp in arguments {
                evaluated_args.push(eval_expression(exp, env)?);
            }

            eval_call_expression(obj, evaluated_args, expression.position)
        }
        Expression::IndexExpression(obj, index) => {
            let obj = eval_expression(obj, env)?;
            let index = eval_expression(index, env)?;
            eval_index_expression(&obj, &index)
                .map_err(|e| MonkeyError::Interpreter(expression.position, e))
        }
        Expression::BlockExpression(block) => eval_block(block, env),
    }
}

pub fn eval_statement(statement: &NodeStatement, env: &EnvHandle) -> MonkeyResult<Object> {
    match &statement.statement {
        Statement::ExpressionStatement(exp) => eval_expression(exp, env),
        Statement::Return(exp) => {
            let value = eval_expression(exp, env)?;
            Err(MonkeyError::Interpreter(
                statement.position,
                RuntimeError::ReturnValue(Box::new(value)),
            ))
        }
        Statement::Let(let_statement) => {
            let (name, exp) = &**let_statement;
            let value = eval_expression(&exp, env)?;
            env.borrow_mut().insert(name.clone(), value);
            Ok(Object::Nil)
        }
    }
}

fn eval_block(block: &[NodeStatement], env: &EnvHandle) -> MonkeyResult<Object> {
    let mut last = Object::Nil;
    let new_env = Rc::new(RefCell::new(Environment::extend(env)));
    for s in block {
        last = eval_statement(s, &new_env)?;
    }
    Ok(last)
}

fn eval_prefix_expression(operator: &Token, right: &Object) -> Result<Object, RuntimeError> {
    match (operator, right) {
        (Token::Minus, Object::Integer(i)) => Ok(Object::Integer(-i)),
        (Token::Bang, obj) => Ok(Object::Boolean(!obj.is_truthy())),
        _ => Err(PrefixTypeError(operator.clone(), right.type_str())),
    }
}

fn eval_infix_expression(
    left: &Object,
    operator: &Token,
    right: &Object,
) -> Result<Object, RuntimeError> {
    match (left, operator, right) {
        // Equality operators
        (l, Token::Equals, r) => Ok(Object::Boolean(Object::eq(l, r).unwrap_or(false))),
        (l, Token::NotEquals, r) => Ok(Object::Boolean(!Object::eq(l, r).unwrap_or(false))),
        // int `anything` int
        (Object::Integer(l), op, Object::Integer(r)) => eval_int_infix_expression(op, *l, *r),
        // String concatenation
        (Object::Str(l), Token::Plus, Object::Str(r)) => {
            let l = (**l).clone();
            let r = &**r;
            Ok(Object::Str(Box::new(l + r)))
        }

        _ => Err(InfixTypeError(
            left.type_str(),
            operator.clone(),
            right.type_str(),
        )),
    }
}

fn eval_int_infix_expression(
    operator: &Token,
    left: i64,
    right: i64,
) -> Result<Object, RuntimeError> {
    match operator {
        // Arithmetic operators
        Token::Plus => Ok(Object::Integer(left + right)),
        Token::Minus => Ok(Object::Integer(left - right)),
        Token::Asterisk => Ok(Object::Integer(left * right)),
        Token::Slash if right == 0 => Err(DivOrModByZero),
        Token::Slash => Ok(Object::Integer(left / right)),
        Token::Exponent if right < 0 => Err(NegativeExponent),
        Token::Exponent => Ok(Object::Integer(left.pow(right as u32))),
        Token::Modulo if right == 0 => Err(DivOrModByZero),
        Token::Modulo => Ok(Object::Integer(left % right)),

        // Comparison operators
        Token::LessThan => Ok(Object::Boolean(left < right)),
        Token::LessEq => Ok(Object::Boolean(left <= right)),
        Token::GreaterThan => Ok(Object::Boolean(left > right)),
        Token::GreaterEq => Ok(Object::Boolean(left >= right)),

        _ => unreachable!(),
    }
}

pub fn eval_call_expression(
    obj: Object,
    args: Vec<Object>,
    call_position: (usize, usize), // We need the caller position to properly report errors
) -> MonkeyResult<Object> {
    match obj {
        Object::InterpreterFunc(fo) => call_function_object(*fo, args, call_position),
        Object::Builtin(b) => {
            b.0(args).map_err(|e| MonkeyError::Interpreter(call_position, e))
        }
        other => Err(MonkeyError::Interpreter(
            call_position,
            NotCallable(other.type_str()),
        )),
    }
}

fn call_function_object(
    fo: InterpreterFunctionObject,
    args: Vec<Object>,
    call_pos: (usize, usize),
) -> MonkeyResult<Object> {
    if fo.parameters.len() != args.len() {
        return Err(MonkeyError::Interpreter(
            call_pos,
            WrongNumberOfArgs(fo.parameters.len(), args.len()),
        ));
    }
    let mut call_env = fo.environment.borrow().clone();
    for (name, value) in fo.parameters.into_iter().zip(args) {
        call_env.insert(name, value);
    }
    let result = eval_block(&fo.body, &Rc::new(RefCell::new(call_env)));
    result.or_else(|e| {
        if let MonkeyError::Interpreter(_, ReturnValue(obj)) = e {
            Ok(*obj)
        } else {
            Err(e)
        }
    })
}

pub fn eval_index_expression(object: &Object, index: &Object) -> Result<Object, RuntimeError> {
    // This function is pub because the "get" built-in needs to call it
    match (object, index) {
        (Object::Array(vector), Object::Integer(i)) => {
            if *i < 0 || *i >= vector.len() as i64 {
                Err(IndexOutOfBounds(*i))
            } else {
                Ok(vector[*i as usize].clone())
            }
        }
        (Object::Array(_), other) => Err(IndexTypeError(other.type_str())),
        (Object::Hash(map), key) => {
            let key_type = key.type_str();
            let key = HashableObject::from_object(key.clone())
                .ok_or_else(|| HashKeyTypeError(key_type))?;
            let value = map.get(&key).ok_or_else(|| KeyError(key))?;
            Ok(value.clone())
        }
        (Object::Str(s), Object::Integer(i)) => {
            let chars = s.chars().collect::<Vec<_>>();
            if *i < 0 || *i >= chars.len() as i64 {
                Err(IndexOutOfBounds(*i))
            } else {
                Ok(Object::Str(Box::new(chars[*i as usize].to_string())))
            }
        }
        (Object::Str(_), other) => Err(IndexTypeError(other.type_str())),
        (other, _) => Err(IndexingWrongType(other.type_str())),
    }
}
