// @TODO: Document this module
use crate::ast::*;
use crate::environment::*;
use crate::error::*;
use crate::object::*;
use crate::token::Token;

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
                None => Err(runtime_err(expression.position, IdenNotFound(s.clone()))),
            }
        }
        Expression::IntLiteral(i) => Ok(Object::Integer(*i)),
        Expression::Boolean(b) => Ok(Object::Boolean(*b)),
        Expression::StringLiteral(s) => Ok(Object::Str(s.clone())),
        Expression::ArrayLiteral(v) => {
            let mut elements = Vec::with_capacity(v.len());
            for exp in v {
                elements.push(eval_expression(exp, env)?);
            }
            Ok(Object::Array(elements))
        }
        Expression::HashLiteral(v) => {
            let mut map = HashMap::new();
            for (key, val) in v {
                let obj = eval_expression(key, env)?;
                let obj_type = obj.type_str();
                let key = match HashableObject::from_object(obj) {
                    Some(v) => v,
                    None => {
                        return Err(runtime_err(expression.position, HashKeyTypeError(obj_type)))
                    }
                };

                let val = eval_expression(val, env)?;
                map.insert(key, val);
            }
            Ok(Object::Hash(map))
        }
        Expression::PrefixExpression(tk, e) => {
            let right_side = eval_expression(e, env)?;
            eval_prefix_expression(tk, &right_side)
                .map_err(|e| runtime_err(expression.position, e))
        }
        Expression::InfixExpression(l, tk, r) => {
            let left_side = eval_expression(l, env)?;
            let right_side = eval_expression(r, env)?;
            eval_infix_expression(&left_side, tk, &right_side)
                .map_err(|e| runtime_err(expression.position, e))
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
                Object::Function(fo) => {
                    call_function_object(fo, evaluated_args, expression.position)
                }
                Object::Builtin(b) => {
                    b.0(evaluated_args, env).map_err(|e| runtime_err(expression.position, e))
                }
                other => Err(runtime_err(
                    expression.position,
                    NotCallable(other.type_str()),
                )),
            }
        }
        Expression::IndexExpression(obj, index) => {
            let obj = eval_expression(obj, env)?;
            let index = eval_expression(index, env)?;
            eval_index_expression(&obj, &index).map_err(|e| runtime_err(expression.position, e))
        }
        Expression::BlockExpression(block) => eval_block(block, env),
    }
}

pub fn eval_statement(statement: &NodeStatement, env: &EnvHandle) -> MonkeyResult<Object> {
    match &statement.statement {
        Statement::ExpressionStatement(exp) => eval_expression(exp, env),
        Statement::BlockStatement(block) => eval_block(block, env),
        Statement::Return(exp) => {
            let value = eval_expression(exp, env)?;
            Err(runtime_err(statement.position, RuntimeError::ReturnValue(Box::new(value))))
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
        (l, Token::Equals, r) => Ok(Object::Boolean(are_equal(l, r))),
        (l, Token::NotEquals, r) => Ok(Object::Boolean(!are_equal(l, r))),
        // int `anything` int
        (Object::Integer(l), op, Object::Integer(r)) => eval_int_infix_expression(op, *l, *r),
        // String concatenation
        (Object::Str(l), Token::Plus, Object::Str(r)) => Ok(Object::Str(l.clone() + r)),

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

fn are_equal(left: &Object, right: &Object) -> bool {
    // Function object, array, and hash comparisons are unsupported, and always return false
    match (left, right) {
        (Object::Integer(l), Object::Integer(r)) => l == r,
        (Object::Boolean(l), Object::Boolean(r)) => l == r,
        (Object::Str(l), Object::Str(r)) => l == r,
        (Object::Nil, Object::Nil) => true,
        _ => false,
    }
}

pub fn call_function_object(
    fo: FunctionObject,
    args: Vec<Object>,
    call_pos: (usize, usize),
) -> MonkeyResult<Object> {
    if fo.parameters.len() != args.len() {
        return Err(runtime_err(
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
        if let ErrorType::Runtime(ReturnValue(obj)) = e.error  {
            Ok(*obj)
        } else {
            Err(e)
        }
    })
}

pub fn eval_index_expression(object: &Object, index: &Object) -> Result<Object, RuntimeError> {
    // This function is pub because the "get" built-in needs to call it
    // @TODO: Implement string indexing
    match (object, index) {
        (Object::Array(vector), Object::Integer(i)) => {
            if *i < 0 || *i >= vector.len() as i64 {
                Err(IndexOutOfBounds(*i))
            } else {
                Ok(vector[*i as usize].clone())
            }
        }
        (Object::Array(_), other) => Err(ArrayIndexTypeError(other.type_str())),
        (Object::Hash(map), key) => {
            let key_type = key.type_str();
            let key = HashableObject::from_object(key.clone())
                .ok_or_else(|| HashKeyTypeError(key_type))?;
            let value = map.get(&key).ok_or_else(|| KeyError(key))?;
            Ok(value.clone())
        }
        (other, _) => Err(IndexingWrongType(other.type_str())),
    }
}

#[cfg(test)]
mod tests {
    // @TODO: Add tests for string operations
    // @TODO: Add tests for block expressions
    use super::*;
    use Object::*;

    fn assert_eval(input: &str, expected: &[Object]) {
        use crate::lexer::Lexer;
        use crate::parser::Parser;

        // Parse program into vector of statements
        let parsed = Parser::new(Lexer::from_string(input.into()).unwrap())
            .unwrap()
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
        let parsed = Parser::new(Lexer::from_string(input.into()).unwrap())
            .unwrap()
            .parse_program()
            .expect("Parser error during test");
        let env = Rc::new(RefCell::new(Environment::empty()));
        for (statement, &error) in parsed.iter().zip(expected_errors) {
            let got = eval_statement(statement, &env).expect_err("No runtime error encountered");
            match got.error {
                ErrorType::Runtime(e) => assert_eq!(e.message(), error),
                _ => panic!("Wrong error type"),
            }
        }
    }

    #[test]
    fn test_int_expressions() {
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
            1 + 2 / 3 ^ 0;
            3 ^ 2 / 3 + 4;
            16 % 7;
            7 % 7;
            -17 % 13;
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
            Integer(3),
            Integer(7),
            Integer(2),
            Integer(0),
            Integer(-4),
        ];
        assert_eval(input, &expected);
    }

    #[test]
    fn test_bool_expressions() {
        let input = r#"
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
            !(-9)
            !0
            !"string"
            !nil
        "#;
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
            Boolean(false),
            Boolean(true),
            Boolean(false),
            Boolean(true),
        ];
        assert_eval(input, &expected);
    }

    #[test]
    fn test_string_operations() {
        let input = r#"
            "abc" + "";
            "" + "abc";
            "abc" + "def";
        "#;
        let expected = [
            Object::Str("abc".into()),
            Object::Str("abc".into()),
            Object::Str("abcdef".into()),
        ];
        assert_eval(input, &expected);
    }

    #[test]
    fn test_blocks() {
        let input = "
            { 5 }
            { 2; false }
            {
                { true; 3; }
            }
            let a = {
                let b = 9;
                b * (b - 1) * (b - 2);
            };
            a;
            let c = 2;
            let d = {
                let c = 3;
                c;
            };
            d;
        ";
        let expected = [
            Integer(5),
            Boolean(false),
            Integer(3),
            Nil,
            Integer(504),
            Nil,
            Nil,
            Integer(3),
        ];
        assert_eval(input, &expected);
    }

    #[test]
    fn test_if_expressions() {
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
    fn test_return_statements() {
        let input = r#"
            fn() { 1; return 0; }();
            fn() { 4; return 1; 9; }();
            fn() { 16; return 2; return 25; 36; }();
            fn() {
                if true {
                    if true {
                        return 3;
                    }
                    return 49;
                }
            }();
            fn() {
                if false {
                    return 64;
                } else {
                    return 4;
                }
            }();
        "#;
        let expected = [Integer(0), Integer(1), Integer(2), Integer(3), Integer(4)];
        assert_eval(input, &expected);
    }
    #[test]
    fn test_return_in_expressions() {
        // One thing that makes return statements very problematic is the fact that they can
        // appear inside block expressions, meaning they can appear in any expression context.
        // The following tests make sure everything works properly in these contexts.
        let input = r#"
            // Inside prefix and infix expressions
            fn() {
                !{ return 0; }
            }();
            fn() {
                0 + { return 1; }
            }();

            // Inside let statements
            fn() {
                let a = { return 2; false; };
            }();

            // Inside if conditions
            fn() {
                if ({ return 3; false; }) {
                    false;
                }
            }();

            // Inside array literals
            fn() {
                [false, { return 4;}, false]
            }();

            // Inside hash literals
            // As values
            fn() {
                #{"a": false, "b": { return 5; }}
            }();
            // As keys
            fn() {
                #{ { return 6; "a" }: false }
            }();

            // Inside indexing expressions
            // As index
            fn() {
                [false, false][{ return 7; }];
            }();
            // As expression being indexed
            fn() {
                { return 8; }[0];
            }();

            // Inside function calls
            // As parameter
            fn() {
                let foo = fn(x) {};
                foo({ return 9; });
            }();
            // As function being called
            fn() {
                ({ return 10; })();
            }();

            // Inside return statements
            // I know this is silly, but it's better to test it anyway
            fn() {
                return { return 11; };
            }();
        "#;
        let expected = [
            Integer(0),
            Integer(1),
            Integer(2),
            Integer(3),
            Integer(4),
            Integer(5),
            Integer(6),
            Integer(7),
            Integer(8),
            Integer(9),
            Integer(10),
            Integer(11),
        ];
        assert_eval(input, &expected);
    }

    #[test]
    fn test_let_statements() {
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
    fn test_arrays() {
        let input = "
            [];
            [0, nil, false];
            [0, [1]];
            let arr = [0, 1, 1, 2, 3, 5, 8, 13];
            arr[5];
            let arr = [[0], [0, 1], [0, 1, 2]];
            arr[2][2];
        ";
        let expected = [
            Array(Vec::new()),
            Array(vec![Integer(0), Nil, Boolean(false)]),
            Array(vec![Integer(0), Array(vec![Integer(1)])]),
            Nil,
            Integer(5),
            Nil,
            Integer(2),
        ];
        assert_eval(input, &expected);
    }

    #[test]
    fn test_hashes() {
        macro_rules! map {
            ($($key:expr => $value:expr),*) => {
                {
                    let mut _map = HashMap::new();
                    $(_map.insert($key, $value);)*
                    _map
                }
            };
        }
        let input = r#"
            #{};
            #{"a": true, "b": [], "c": 3};
            #{"nested": #{}};
            let h = #{
                "something": nil,
                1 + 2: 5 - 1,
                !true: "indeed"
            };
            h;
            h["something"];
            h[3];
            h[false];
        "#;
        let expected = [
            Hash(map! {}),
            Hash(map! {
                HashableObject::Str("a".into()) => Object::Boolean(true),
                HashableObject::Str("b".into()) => Object::Array(Vec::new()),
                HashableObject::Str("c".into()) => Object::Integer(3)
            }),
            Hash(map! { HashableObject::Str("nested".into()) =>  Hash(map! {}) }),
            Nil,
            Hash(map! {
                HashableObject::Str("something".into()) => Object::Nil,
                HashableObject::Integer(3) => Object::Integer(4),
                HashableObject::Boolean(false) => Object::Str("indeed".into())
            }),
            Object::Nil,
            Object::Integer(4),
            Object::Str("indeed".into()),
        ];
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
            Integer(289),
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
            "identifier not found: 'a'",
            "'nil' is not a function object",
            "'int' is not a function object",
            "`return` outside of function context",
            "wrong number of arguments: expected 0 arguments but 2 were given",
        ];
        assert_runtime_error(input, &expected);

        // Prefix expressions
        let input = "
            -true
            -(fn(){})
            -nil
        ";
        let expected = [
            "unsuported operand type for prefix operator `-`: 'bool'",
            "unsuported operand type for prefix operator `-`: 'function'",
            "unsuported operand type for prefix operator `-`: 'nil'",
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
            "unsuported operand types for infix operator `+`: 'bool' and 'bool'",
            "unsuported operand types for infix operator `<`: 'bool' and 'bool'",
            "unsuported operand types for infix operator `/`: 'bool' and 'nil'",
            "unsuported operand types for infix operator `>=`: 'function' and 'bool'",
            "unsuported operand types for infix operator `>`: 'bool' and 'nil'",
            "unsuported operand types for infix operator `*`: 'function' and 'function'",
        ];
        assert_runtime_error(input, &expected);

        // Arithmetic errors
        let input = "
            2 / 0;
            2 % 0;
            2 ^ (-1)
        ";
        let expected = [
            "division or modulo by zero",
            "division or modulo by zero",
            "negative exponent",
        ];
        assert_runtime_error(input, &expected);
    }
}
