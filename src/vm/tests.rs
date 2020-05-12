use super::*;
use crate::test_utils;

fn assert_vm_runs(input: &[&str], expected: &[Object]) {
    for (program, exp) in input.iter().zip(expected) {
        let bytecode =
            test_utils::parse_and_compile(program).expect("Parser or compiler error during test");
        let mut vm = VM::new();
        vm.run(bytecode).unwrap();
        assert!(test_utils::compare_vm_objects(exp, vm.stack_top().unwrap()));
    }
}

#[test]
fn test_integer_arithmetic() {
    let input = ["2 + 3", "-3"];
    let expected = [Object::Integer(5), Object::Integer(-3)];
    assert_vm_runs(&input, &expected);
}

#[test]
fn test_boolean_expressions() {
    let input = [
        "true",
        "false",
        "2 >= 3 == true",
        "false != 1 < 2",
        "!false",
        "!(if false { 3 })",
    ];
    let expected = [
        Object::Boolean(true),
        Object::Boolean(false),
        Object::Boolean(false),
        Object::Boolean(true),
        Object::Boolean(true),
        Object::Boolean(true),
    ];
    assert_vm_runs(&input, &expected);
}

#[test]
fn test_conditional_expressions() {
    let input = [
        "if true { 10 }",
        "if true { 10 } else { 20 }",
        "if false { 10 } else { 20 }",
        "if 1 > 2 { 10 } else { 20 }",
        "if 1 > 2 { 10 }",
    ];
    let expected = [
        Object::Integer(10),
        Object::Integer(10),
        Object::Integer(20),
        Object::Integer(20),
        Object::Nil,
    ];
    assert_vm_runs(&input, &expected);
}

#[test]
fn test_global_assignment() {
    let input = [
        "let one = 1; one",
        "let one = 1; let two = 2; one + two",
        "let one = 1; let two = one + one; one + two",
    ];
    let expected = [Object::Integer(1), Object::Integer(3), Object::Integer(3)];
    assert_vm_runs(&input, &expected);
}

#[test]
fn test_strings() {
    let input = [
        r#""monkey""#,
        r#""mon" + "key""#,
        r#""mon" + "key" + "banana""#,
    ];
    let expected = [
        Object::Str("monkey".into()),
        Object::Str("monkey".into()),
        Object::Str("monkeybanana".into()),
    ];
    assert_vm_runs(&input, &expected);
}

#[test]
fn test_arrays() {
    let input = ["[]", "[1, 2, 3]", "[1 + 2, 3 - 4, 5 * 6]"];
    let expected = [
        Object::Array(vec![]),
        Object::Array(vec![
            Object::Integer(1),
            Object::Integer(2),
            Object::Integer(3),
        ]),
        Object::Array(vec![
            Object::Integer(3),
            Object::Integer(-1),
            Object::Integer(30),
        ]),
    ];
    assert_vm_runs(&input, &expected);
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
    let input = ["#{}", "#{ 1: 2, 2: 3 }", "#{ 1 + 1: 2 * 2, 3 + 3: 4 * 4 }"];
    let expected = [
        Object::Hash(map! {}),
        Object::Hash(map! {
            HashableObject::Integer(1) => Object::Integer(2),
            HashableObject::Integer(2) => Object::Integer(3)
        }),
        Object::Hash(map! {
            HashableObject::Integer(2) => Object::Integer(4),
            HashableObject::Integer(6) => Object::Integer(16)
        }),
    ];
    assert_vm_runs(&input, &expected)
}

#[test]
fn test_index_expressions() {
    let input = [
        "[1, 2, 3][1]",
        "[1, 2, 3][0 + 2]",
        "[[1, 1, 1]][0][0]",
        "#{ 1: 1, 2: 2 }[1]",
        "#{ 1: 1, 2: 2 }[2]",
    ];
    let expected = [
        Object::Integer(2),
        Object::Integer(3),
        Object::Integer(1),
        Object::Integer(1),
        Object::Integer(2),
    ];
    assert_vm_runs(&input, &expected);
}

#[test]
fn test_function_calls() {
    let input = [
        "let foo = fn() { 5 + 10; }; foo()",
        "let foo = fn() { return 5 + 10; }; foo()",
        "
            let one = fn() { 1 };
            let two = fn() { 1 + one() };
            let three = fn() { two() + one() };
            (one() + three()) * two();
        ",
        "let nothing = fn() {}; nothing()",
    ];
    let expected = [
        Object::Integer(15),
        Object::Integer(15),
        Object::Integer(8),
        Object::Nil,
    ];
    assert_vm_runs(&input, &expected);
}

#[test]
fn test_stack_cleaning_after_call() {
    let input = "
        let foo = fn() {
            5 + (if true { return 2; })
        };
        foo();
    ";
    let bytecode =
        test_utils::parse_and_compile(input).expect("Parser or compiler error during test");
    let mut vm = VM::new();
    vm.run(bytecode).unwrap();
    assert_eq!(vm.stack.len(), 1);
}

#[test]
fn test_local_bindings() {
    let input = [
        "let one = fn() { let one = 1; one }; one()",
        "let one_and_two = fn() { let one = 1; let two = 2; one + two; }; one_and_two();",
        "let one_and_two = fn() { let one = 1; let two = 2; one + two; };
        let three_and_four = fn() { let three = 3; let four = 4; three + four; };
        one_and_two() + three_and_four();",
        "let first_foobar = fn() { let foobar = 50; foobar; };
        let second_foobar = fn() { let foobar = 100; foobar; };
        first_foobar() + second_foobar();",
        "let global_seed = 50;
        let minus_one = fn() {
            let num = 1;
            global_seed - num;
        };
        let minus_two = fn() {
            let num = 2;
            global_seed - num;
        };
        minus_one() + minus_two();",
    ];
    let expected = [
        Object::Integer(1),
        Object::Integer(3),
        Object::Integer(10),
        Object::Integer(150),
        Object::Integer(97),
    ];
    assert_vm_runs(&input, &expected);
}

#[test]
fn test_function_arguments() {
    let input = [
        "let id = fn(x) { x }; id(4)",
        "let sum = fn(a, b) { a + b }; sum(1, 2)",
    ];
    let expected = [
        Object::Integer(4),
        Object::Integer(3),
    ];
    assert_vm_runs(&input, &expected);
}
