// @WIP: This whole module is a work in progress, expect function signatures to change
// @TODO: Add proper error handling. Currently, all evaluation functions panic instead of returning
// errors
use crate::ast::*;
use crate::object::*;
use crate::token::Token;

use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct RuntimeError(String);

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for RuntimeError {}

macro_rules! runtime_err {
    ($($arg:expr),*) => { Err(RuntimeError(format!($($arg),*))) }
}

pub type EvalResult = Result<Object, RuntimeError>;

pub fn eval_expression(expression: Expression) -> EvalResult {
    match expression {
        Expression::IntLiteral(i) => Ok(Object::Integer(i)),
        Expression::Boolean(b) => Ok(Object::Boolean(b)),
        Expression::PrefixExpression(tk, e) => {
            let right_side = eval_expression(*e)?;
            eval_prefix_expression(tk, right_side)
        }
        Expression::InfixExpression(l, tk, r) => {
            let left_side = eval_expression(*l)?;
            let right_side = eval_expression(*r)?;
            eval_infix_expression(tk, left_side, right_side)
        }
        Expression::IfExpression { condition, consequence, alternative } => {
            let value = eval_expression(*condition)?;
            if is_truthy(value) {
                eval_block(consequence)
            } else {
                eval_block(alternative)
            }
        }
        _ => panic!("Expression type still not supported"),
    }
}

pub fn eval_statement(statement: Statement) -> EvalResult {
    match statement {
        Statement::ExpressionStatement(exp) => eval_expression(*exp),
        Statement::BlockStatement(block) => eval_block(block),
        // @WIP: Return statements are currently a work in progress. For now, the evaluator never
        // unwraps the `ReturnValue` objects. When I implement functions and function calling, I
        // will properly implement return values as well. I'm considering not allowing return
        // statements outside function bodies, partly because it makes the implementation simpler,
        // but also because I feel it's unecessary to language.
        Statement::Return(exp) => {
            let value = eval_expression(*exp)?;
            Ok(Object::ReturnValue(Box::new(value)))
        }
        _ => panic!("Statement type still not supported"),
    }
}

fn eval_block(block: Vec<Statement>) -> EvalResult {
    let mut last = Object::Nil;
    for s in block {
        last = eval_statement(s)?;
        if let Object::ReturnValue(_) = &last {
            return Ok(last);
        }
    }
    Ok(last)
}

fn eval_prefix_expression(operator: Token, right: Object) -> EvalResult {
    match (operator, right) {
        (Token::Minus, Object::Integer(i)) => Ok(Object::Integer(-i)),
        (Token::Bang, obj) => Ok(Object::Boolean(!is_truthy(obj))),
        (op, r) => runtime_err!(
            "Unsuported operand type for prefix operator {}: '{}'",
            op.type_str(), r.type_str()
        ),
    }
}

fn eval_infix_expression(operator: Token, left: Object, right: Object) -> EvalResult {
    match (left, operator, right) {
        // int `anything` int
        (Object::Integer(l), op, Object::Integer(r)) => eval_int_infix_expression(op, l, r),
        // bool == bool
        (Object::Boolean(l), Token::Equals, Object::Boolean(r)) => Ok(Object::Boolean(l == r)),
        // bool != bool
        (Object::Boolean(l), Token::NotEquals, Object::Boolean(r)) => Ok(Object::Boolean(l != r)),

        (l, op, r) => runtime_err!(
            "Unsuported operand types for operator {}: '{}' and '{}'",
            op.type_str(), l.type_str(), r.type_str()
        ),
    }
}

fn eval_int_infix_expression(operator: Token, left: i64, right: i64) -> EvalResult {
    match operator {
        // Arithmetic operators
        Token::Plus => Ok(Object::Integer(left + right)),
        Token::Minus => Ok(Object::Integer(left - right)),
        Token::Asterisk => Ok(Object::Integer(left * right)),
        Token::Slash => Ok(Object::Integer(left / right)),

        // Comparison operators
        Token::Equals => Ok(Object::Boolean(left == right)),
        Token::NotEquals => Ok(Object::Boolean(left != right)),
        Token::LessThan => Ok(Object::Boolean(left < right)),
        Token::LessEq => Ok(Object::Boolean(left <= right)),
        Token::GreaterThan => Ok(Object::Boolean(left > right)),
        Token::GreaterEq => Ok(Object::Boolean(left >= right)),

        _ => runtime_err!(
            "Unsuported operand types for operator {}: 'int' and 'int'",
            operator.type_str()
        ),
    }
}

fn is_truthy(obj: Object) -> bool {
    match obj {
        Object::Boolean(b) => b,
        Object::Nil => false,
        // I am unsure if I want integer values to have a truth value or not. For now, I will stick
        // to the book, which specifies that they do
        Object::Integer(i) => i != 0,
        _ => panic!(), // @DEBUG
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Object::*;

    fn assert_eval(input: &str, expected: &[Object]) {
        use crate::lexer::Lexer;
        use crate::parser::Parser;

        // Parse program into vector of statements
        let parsed = Parser::new(Lexer::new(input.into()))
            .parse_program()
            .expect("Parser error during test");

        assert_eq!(parsed.len(), expected.len());

        // Eval program statements and compare with expected
        for (st, exp) in parsed.into_iter().zip(expected) {
            let got = eval_statement(st).expect("Runtime error during test");
            assert_eq!(&got, exp);
        }
    }

    #[test]
    fn test_eval_int_expression() {
        let input = "
            5;
            -10;
            --42;
            -0;
            2 + 2;
            1 * 2 + 3;
            1 + 2 * 3;
            (1 + 1) * (2 + 2);
            66 / (2 * 3 + 5);
        ";
        let expected = [
            Integer(5),
            Integer(-10),
            Integer(42),
            Integer(0),
            Integer(4),
            Integer(5),
            Integer(7),
            Integer(8),
            Integer(6),
        ];
        assert_eval(input, &expected);
    }

    #[test]
    fn test_eval_bool_expression() {
        let input = "
            false;
            !true;
            !!true;
            1 < 2;
            2 <= 0;
            1 > 2;
            2 >= 0;
            0 == 0;
            1 != 0;
            true == true;
            false == false;
            false != false;
            true != false;
        ";
        let expected = [
            Boolean(false),
            Boolean(false),
            Boolean(true),
            Boolean(true),
            Boolean(false),
            Boolean(false),
            Boolean(true),
            Boolean(true),
            Boolean(true),
            Boolean(true),
            Boolean(true),
            Boolean(false),
            Boolean(true),
        ];
        assert_eval(input, &expected);
    }

    #[test]
    fn test_eval_block_statement() {
        let input = "
            { 5 }
            { 2; false }
            {
                { true; 3; }
            }
        ";
        let expected = [Integer(5), Boolean(false), Integer(3)];
        assert_eval(input, &expected);
    }

    #[test]
    fn test_eval_if_expression() {
        let input = "
            if true { 10 }
            if false { 10 }
            if 1 { 10 }
            if 0 { 10 }
            if 2 < 5 { 10 }
            if true { 10 } else { 20 }
            if false { 10 } else { 20 }
        ";
        let expected = [
            Integer(10),
            Nil,
            Integer(10),
            Nil,
            Integer(10),
            Integer(10),
            Integer(20),
        ];
        assert_eval(input, &expected);
    }

    #[test]
    fn test_eval_return_statement() {
        // @WIP
        let input = "
            { return 5 }
            { 5; return 10 }
            { 4; return 9; 3 }
            { 8; return 6; return 0; 2 }
            if true { if true { return 1; } return 2; }
        ";
        let expected = [
            ReturnValue(Box::new(Integer(5))),
            ReturnValue(Box::new(Integer(10))), 
            ReturnValue(Box::new(Integer(9))),
            ReturnValue(Box::new(Integer(6))),
            ReturnValue(Box::new(Integer(1)))
        ];
        assert_eval(input, &expected);
    }
}
