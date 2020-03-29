// @TODO: Document this module
use crate::ast::*;
use crate::environment::*;
use crate::object::*;
use crate::token::Token;

use std::cell::RefCell;
use std::error::Error;
use std::fmt;
use std::rc::Rc;

// @TODO: Make `RuntimeError` an enum
#[derive(Debug, PartialEq)]
pub struct RuntimeError(pub String);

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for RuntimeError {}

#[macro_export]
macro_rules! runtime_err {
    ($($arg:expr),*) => { Err(RuntimeError(format!($($arg),*))) }
}

pub type EvalResult = Result<Object, RuntimeError>;

pub fn run_program(program: Vec<Statement>) -> Result<(), RuntimeError> {
    let env = Rc::new(RefCell::new(Environment::empty()));
    for statement in program {
        eval_statement(&statement, &env)?;
    }
    Ok(())
}

pub fn eval_expression(expression: &Expression, env: &EnvHandle) -> EvalResult {
    match expression {
        Expression::Identifier(s) => {
            // Note: This clones the object
            match env.borrow().get(&s) {
                Some(value) => Ok(value),
                None => runtime_err!("Identifier not found: '{}'", s),
            }
        }
        Expression::IntLiteral(i) => Ok(Object::Integer(*i)),
        Expression::Boolean(b) => Ok(Object::Boolean(*b)),
        Expression::StringLiteral(s) => Ok(Object::Str(s.clone())),
        Expression::PrefixExpression(tk, e) => {
            let right_side = eval_expression(e, env)?;
            eval_prefix_expression(tk, &right_side)
        }
        Expression::InfixExpression(l, tk, r) => {
            let left_side = eval_expression(l, env)?;
            let right_side = eval_expression(r, env)?;
            eval_infix_expression(tk, &left_side, &right_side)
        }
        Expression::IfExpression { condition, consequence, alternative } => {
            let value = eval_expression(condition, env)?;
            if value.is_truthy() {
                eval_block(consequence, env)
            } else {
                eval_block(alternative, env)
            }
        }
        Expression::Nil => Ok(Object::Nil),
        Expression::FunctionLiteral { parameters, body } => {
            let fo = FunctionObject {
                environment: Rc::clone(env),
                parameters: parameters.clone(),
                body: body.clone(),
            };
            Ok(Object::Function(fo))
        }
        Expression::CallExpression { function, arguments } => {
            // Evaluate the called object and make sure it's a function
            let obj = eval_expression(function, env)?;
            // Evaluate all arguments sequentially
            let mut evaluated_args = Vec::with_capacity(arguments.len());
            for exp in arguments {
                evaluated_args.push(eval_expression(exp, env)?);
            }
            
            match obj {
                Object::Function(fo) => call_function_object(fo, evaluated_args),
                Object::Builtin(fp) => fp(evaluated_args),
                other => runtime_err!("'{}' is not a function object", other.type_str()),
            }
        }
    }
}

pub fn eval_statement(statement: &Statement, env: &EnvHandle) -> EvalResult {
    match statement {
        Statement::ExpressionStatement(exp) => eval_expression(exp, env),
        Statement::BlockStatement(block) => eval_block(block, env),
        Statement::Return(exp) => {
            if !env.borrow().is_fn_context {
                return runtime_err!("`return` outside function context");
            }
            let value = eval_expression(exp, env)?;
            Ok(Object::ReturnValue(Box::new(value)))
        }
        Statement::Let(let_statement) => {
            let (name, exp) = &**let_statement;
            let value = eval_expression(&exp, env)?;
            env.borrow_mut().insert(name.clone(), value);
            Ok(Object::Nil)
        }
    }
}

fn eval_block(block: &[Statement], env: &EnvHandle) -> EvalResult {
    let mut last = Object::Nil;
    let new_env = Rc::new(RefCell::new(Environment::extend(env)));
    for s in block {
        last = eval_statement(s, &new_env)?;
        if let Object::ReturnValue(_) = &last {
            return Ok(last);
        }
    }
    Ok(last)
}

fn eval_prefix_expression(operator: &Token, right: &Object) -> EvalResult {
    match (operator, right) {
        (Token::Minus, Object::Integer(i)) => Ok(Object::Integer(-i)),
        (Token::Bang, obj) => Ok(Object::Boolean(!obj.is_truthy())),
        (op, r) => runtime_err!(
            "Unsuported operand type for prefix operator {}: '{}'",
            op.type_str(),
            r.type_str()
        ),
    }
}

fn eval_infix_expression(operator: &Token, left: &Object, right: &Object) -> EvalResult {
    match (left, operator, right) {
        // Equality operators
        (l, Token::Equals, r) => Ok(Object::Boolean(are_equal(l, r))),
        (l, Token::NotEquals, r) => Ok(Object::Boolean(!are_equal(l, r))),
        // int `anything` int
        (Object::Integer(l), op, Object::Integer(r)) => eval_int_infix_expression(op, *l, *r),
        // String concatenation
        (Object::Str(l), Token::Plus, Object::Str(r)) => Ok(Object::Str(l.clone() + r)),

        (l, op, r) => runtime_err!(
            "Unsuported operand types for operator {}: '{}' and '{}'",
            op.type_str(),
            l.type_str(),
            r.type_str()
        ),
    }
}

fn eval_int_infix_expression(operator: &Token, left: i64, right: i64) -> EvalResult {
    match operator {
        // Arithmetic operators
        Token::Plus => Ok(Object::Integer(left + right)),
        Token::Minus => Ok(Object::Integer(left - right)),
        Token::Asterisk => Ok(Object::Integer(left * right)),
        Token::Slash => Ok(Object::Integer(left / right)),

        // Comparison operators
        Token::LessThan => Ok(Object::Boolean(left < right)),
        Token::LessEq => Ok(Object::Boolean(left <= right)),
        Token::GreaterThan => Ok(Object::Boolean(left > right)),
        Token::GreaterEq => Ok(Object::Boolean(left >= right)),

        _ => panic!(), // This is currently unreacheable
    }
}

fn are_equal(left: &Object, right: &Object) -> bool {
    // Funciton object comparison are currently unsupported, and always return false
    match (left, right) {
        (Object::Integer(l), Object::Integer(r)) => l == r,
        (Object::Boolean(l), Object::Boolean(r)) => l == r,
        (Object::Str(l), Object::Str(r)) => l == r,
        (Object::Nil, Object::Nil) => true,
        (_, Object::ReturnValue(_)) => panic!(),
        (Object::ReturnValue(_), _) => panic!(),
        _ => false,
    }
}

fn call_function_object(fo: FunctionObject, args: Vec<Object>) -> EvalResult {
    if fo.parameters.len() != args.len() {
        return runtime_err!(
            "Wrong number of arguments. Expected {} arguments, {} were given",
            fo.parameters.len(),
            args.len()
        );
    }
    let mut call_env = fo.environment.borrow().clone();
    call_env.is_fn_context = true;
    for (name, value) in fo.parameters.into_iter().zip(args) {
        call_env.insert(name, value);
    }
    let result = eval_block(&fo.body, &Rc::new(RefCell::new(call_env)))?;
    Ok(result.unwrap_return_value())
}

#[cfg(test)]
mod tests {
    // @TODO: Add tests for string operations
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
        let env = Rc::new(RefCell::new(Environment::empty()));

        // Eval program statements and compare with expected
        for (st, exp) in parsed.into_iter().zip(expected) {
            let got = eval_statement(&st, &env).expect("Runtime error during test");
            assert_eq!(format!("{}", got), format!("{}", exp));
        }
    }

    fn assert_runtime_error(input: &str, expected_errors: &[&str]) {
        use crate::lexer::Lexer;
        use crate::parser::Parser;

        // Parse program into vector of statements
        let parsed = Parser::new(Lexer::new(input.into()))
            .parse_program()
            .expect("Parser error during test");
        let env = Rc::new(RefCell::new(Environment::empty()));
        for (statement, &error) in parsed.iter().zip(expected_errors) {
            let got = eval_statement(statement, &env).expect_err("No runtime error encountered");
            assert_eq!(got, RuntimeError(error.into()));
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
        let input = "
            fn() { return 5 }()
            fn() { 5; return 10 }()
            fn() { 4; return 9; 3 }()
            fn() { 8; return 6; return 0; 2 }()
            fn() { if true { if true { return 1; } return 2; } }()
        ";
        let expected = [Integer(5), Integer(10), Integer(9), Integer(6), Integer(1)];
        assert_eval(input, &expected);
    }

    #[test]
    fn test_eval_let_statement() {
        let input = "
            { let a = 5; a }
            { let a = 5 * 5; a }
            { let a = 5; let b = a; b }
            { let a = 5; let b = a; let c = a + b + 5; c }
            { let a = 5; { let a = 0 } a }
        ";
        let expected = [Integer(5), Integer(25), Integer(5), Integer(15), Integer(5)];
        assert_eval(input, &expected);
    }

    #[test]
    fn test_functions() {
        let input = "
            let id = fn(x) { x }
            id(5)

            let neg = fn(x) { -x }
            neg(10)

            let sqr = fn(x) { x * x }
            sqr(17)

            let and = fn(p, q) {
                if p {
                    q
                } else {
                    false
                }
            }
            and(false, true)
            and(true, false)
            and(true, true)

            let or = fn(p, q) {
                if p {
                    true
                } else {
                    q
                }
            }
            or(false, true)
            or(true, false)
            or(false, false)

            fn(n) { n + 5 }(3)
            fn(n, m) { n / m }(57, 19)

            let compose = fn(f, g) {
                fn(x) {
                    f(g(x))
                }
            }
            compose(neg, sqr)(17)

            let flip = fn(f) {
                fn(x, y) {
                    f(y, x)
                }
            }
            flip(compose)(neg, sqr)(17)
        ";
        let expected = [
            Nil,
            Integer(5),
            Nil,
            Integer(-10),
            Nil,
            Integer(289),
            Nil,
            Boolean(false),
            Boolean(false),
            Boolean(true),
            Nil,
            Boolean(true),
            Boolean(true),
            Boolean(false),
            Integer(8),
            Integer(3),
            Nil,
            Integer(-289),
            Nil,
            Integer(289)
        ];
        assert_eval(input, &expected);
    }

    #[test]
    fn test_closures() {
        let input = "
            let make_adder = fn(x) {
                let adder = fn(y) { x + y }
                return adder;
            }
            let add_3 = make_adder(3);
            add_3(5);

            let foo = fn() {
                let outer = 1;
                {
                    let inner = 2
                    return fn() { outer + inner };
                }
            }
            foo()()
        ";
        let expected = [Nil, Nil, Integer(8), Nil, Integer(3)];
        assert_eval(input, &expected);
    }

    #[test]
    fn test_recursion() {
        let input = "
            let accumulate = fn(n) {
                if n <= 0 {
                    0
                } else {
                    1 + accumulate(n - 1)
                }
            }
            accumulate(50)

            let fib = fn(n) {
                if n <= 1 {
                    n
                } else {
                    fib(n - 1) + fib(n - 2)
                }
            }
            fib(13)

            let makefact = fn(multiplier) {
                let foo = fn(x) {
                    if x < 1 {
                        multiplier
                    } else {
                        foo(x - 1) * x
                    }
                }
                return foo
            }
            let fact = makefact(2)
            fact(6)
        ";
        let expected = [Nil, Integer(50), Nil, Integer(233), Nil, Nil, Integer(1440)];
        assert_eval(input, &expected);
    }

    #[test]
    fn test_runtime_errors() {
        // Basic errors
        let input = "
            a + b
            nil()
            { let foo = 3; foo() }
            return 2
            { let a = fn(){}; a(1, 2) }
        ";
        let expected = [
            "Identifier not found: 'a'",
            "'nil' is not a function object",
            "'int' is not a function object",
            "`return` outside function context",
            "Wrong number of arguments. Expected 0 arguments, 2 were given",
        ];
        assert_runtime_error(input, &expected);

        // Prefix expressions
        let input = "
            -true
            -(fn(){})
            -nil
        ";
        let expected = [
            "Unsuported operand type for prefix operator `-`: 'bool'",
            "Unsuported operand type for prefix operator `-`: 'function'",
            "Unsuported operand type for prefix operator `-`: 'nil'",
        ];
        assert_runtime_error(input, &expected);

        // Infix expressions
        let input = "
            true + false
            false < false
            true / nil
            fn(){} >= false
            true > nil
            fn(){} * fn(){}
        ";
        let expected = [
            "Unsuported operand types for operator `+`: 'bool' and 'bool'",
            "Unsuported operand types for operator `<`: 'bool' and 'bool'",
            "Unsuported operand types for operator `/`: 'bool' and 'nil'",
            "Unsuported operand types for operator `>=`: 'function' and 'bool'",
            "Unsuported operand types for operator `>`: 'bool' and 'nil'",
            "Unsuported operand types for operator `*`: 'function' and 'function'",
        ];
        assert_runtime_error(input, &expected);
    }
}
