use super::*;
use crate::parser;
use crate::test_utils;
use Object::*;

fn assert_eval(input: &str, expected: &[object::Object]) {
    let parsed = parser::parse(input.into()).expect("Parser error during test");
    assert_eq!(parsed.len(), expected.len());
    let env = Rc::new(RefCell::new(environment::Environment::empty()));

    // Eval program statements and compare with expected
    for (statement, exp) in parsed.into_iter().zip(expected) {
        let got = eval_statement(&statement, &env).expect("Runtime error during test");
        assert!(test_utils::compare_objects(exp, &got));
    }
}

fn assert_runtime_error(input: &str, expected_errors: &[&str]) {
    let parsed = parser::parse(input.into()).expect("Parser error during test");
    let env = Rc::new(RefCell::new(environment::Environment::empty()));
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
        !(-9);
        !0;
        !"string";
        !nil;
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
        "foo"[0];
        "foobar"[5];
    "#;
    let expected = [
        Object::Str("abc".into()),
        Object::Str("abc".into()),
        Object::Str("abcdef".into()),
        Object::Str("f".into()),
        Object::Str("r".into()),
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
        fn() { 81; return; 100; }();
    "#;
    let expected = [
        Integer(0),
        Integer(1),
        Integer(2),
        Integer(3),
        Integer(4),
        Nil,
    ];
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
        { let a = 5; { let a = 0; } a }
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
        let id = fn(x) { x };
        id(5);

        let neg = fn(x) { -x };
        neg(10);

        let sqr = fn(x) { x * x };
        sqr(17);

        let and = fn(p, q) {
            if p {
                q
            } else {
                false
            }
        };
        and(false, true);
        and(true, false);
        and(true, true);

        let or = fn(p, q) {
            if p {
                true
            } else {
                q
            }
        };
        or(false, true);
        or(true, false);
        or(false, false);

        fn(n) { n + 5 }(3);
        fn(n, m) { n / m }(57, 19);

        let compose = fn(f, g) {
            fn(x) {
                f(g(x))
            }
        };
        compose(neg, sqr)(17);

        let flip = fn(f) {
            fn(x, y) {
                f(y, x)
            }
        };
        flip(compose)(neg, sqr)(17);
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
            let adder = fn(y) { x + y };
            return adder;
        };
        let add_3 = make_adder(3);
        add_3(5);

        let foo = fn() {
            let outer = 1;
            {
                let inner = 2;
                return fn() { outer + inner };
            }
        };
        foo()();
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
        };
        accumulate(50);

        let fib = fn(n) {
            if n <= 1 {
                n
            } else {
                fib(n - 1) + fib(n - 2)
            }
        };
        fib(13);

        let makefact = fn(multiplier) {
            let foo = fn(x) {
                if x < 1 {
                    multiplier
                } else {
                    foo(x - 1) * x
                }
            };
            return foo;
        };
        let fact = makefact(2);
        fact(6);
    ";
    let expected = [Nil, Integer(50), Nil, Integer(233), Nil, Nil, Integer(1440)];
    assert_eval(input, &expected);
}

#[test]
fn test_runtime_errors() {
    // Basic errors
    let input = "
        a + b;
        nil();
        { let foo = 3; foo() }
        return 2;
        { let a = fn(){}; a(1, 2) }
    ";
    let expected = [
        "identifier not found: 'a'",
        "'nil' is not a function object or built-in function",
        "'int' is not a function object or built-in function",
        "`return` outside of function context",
        "wrong number of arguments: expected 0 arguments but 2 were given",
    ];
    assert_runtime_error(input, &expected);

    // Prefix expressions
    let input = "
        -true;
        -(fn(){});
        -nil;
    ";
    let expected = [
        "unsuported operand type for prefix operator `-`: 'bool'",
        "unsuported operand type for prefix operator `-`: 'function'",
        "unsuported operand type for prefix operator `-`: 'nil'",
    ];
    assert_runtime_error(input, &expected);

    // Infix expressions
    let input = "
        true + false;
        false < false;
        true / nil;
        fn(){} >= false;
        true > nil;
        fn(){} * fn(){};
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
        2 ^ (-1);
    ";
    let expected = [
        "division or modulo by zero",
        "division or modulo by zero",
        "negative exponent",
    ];
    assert_runtime_error(input, &expected);
}
