use super::*;
use crate::test_utils;

fn assert_vm_runs(input: &[&str], expected: &[Object]) {
    for (program, exp) in input.iter().zip(expected) {
        let bytecode =
            test_utils::parse_and_compile(program).expect("Parser or compiler error during test");
        let mut vm = VM::new();
        vm.run(bytecode).unwrap();
        assert_eq!(exp, &vm.pop().unwrap());
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
        Object::from("monkey"),
        Object::from("monkey"),
        Object::from("monkeybanana"),
    ];
    assert_vm_runs(&input, &expected);
}

#[test]
fn test_arrays() {
    let input = ["[]", "[1, 2, 3]", "[1 + 2, 3 - 4, 5 * 6]"];
    let expected = [
        monkey_array![],
        monkey_array![Object::Integer(1), Object::Integer(2), Object::Integer(3)],
        monkey_array![Object::Integer(3), Object::Integer(-1), Object::Integer(30)],
    ];
    assert_vm_runs(&input, &expected);
}

#[test]
fn test_hashes() {
    let input = ["#{}", "#{ 1: 2, 2: 3 }", "#{ 1 + 1: 2 * 2, 3 + 3: 4 * 4 }"];
    let expected = [
        monkey_hash! {},
        monkey_hash! {
            HashableObject::Integer(1) => Object::Integer(2),
            HashableObject::Integer(2) => Object::Integer(3)
        },
        monkey_hash! {
            HashableObject::Integer(2) => Object::Integer(4),
            HashableObject::Integer(6) => Object::Integer(16)
        },
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
        "let global_num = 10;
        let sum = fn(a, b) {
            let c = a + b;
            c + global_num;
        };
        let outer = fn() {
            sum(1, 2) + sum(3, 4) + global_num;
        };
        outer() + global_num;",
    ];
    let expected = [Object::Integer(4), Object::Integer(3), Object::Integer(50)];
    assert_vm_runs(&input, &expected);
}

#[test]
fn test_builtin_functions() {
    let input = [
        "len(\"\")",
        "len(\"four\")",
        "len(\"hello world\")",
        "len([1, 2, 3])",
        "len([])",
        "puts(\"hi\")",
        "head([1, 2, 3, 4])",
        "head([])",
        "tail([1, 2, 3, 4])",
        "tail([])",
        "tail([0])",
        "push([], 0)",
        "push([1, 2], 0)",
    ];
    let expected = [
        Object::Integer(0),
        Object::Integer(4),
        Object::Integer(11),
        Object::Integer(3),
        Object::Integer(0),
        Object::Nil,
        Object::Integer(1),
        Object::Nil,
        monkey_array![Object::Integer(2), Object::Integer(3), Object::Integer(4)],
        Object::Nil,
        monkey_array![],
        monkey_array![Object::Integer(0)],
        monkey_array![Object::Integer(1), Object::Integer(2), Object::Integer(0)],
    ];
    assert_vm_runs(&input, &expected);
}

#[test]
fn test_closures() {
    let input = [
        "fn(a) { fn(b) { fn(c) { a + b + c } } }(1)(3)(5)",

        "let make_closure = fn(a) {
            fn() { a }
        };
        let closure = make_closure(4);
        closure()",

        "let new_adder = fn(a, b) {
            fn(c) { a + b + c }
        };
        new_adder(1, 2)(8)",
        "let new_adder = fn(a, b) {
            let c = a + b;
            fn(d) { c + d }
        };
        new_adder(1, 2)(8)",

        "let new_adder_outer = fn(a, b) {
            let c = a + b;
            fn(d) {
                let e = d + c;
                fn(f) { e + f }
            }
        };
        let new_adder_inner = new_adder_outer(1, 2);
        let adder = new_adder_inner(3);
        adder(8);",

        "let a = 1;
        let new_adder_outer = fn(b) {
            fn(c) { fn(d) { a + b + c + d } }
        };
        let new_adder_inner = new_adder_outer(2);
        let adder = new_adder_inner(3);
        adder(8);",

        "let new_closure = fn(a, b) {
            let first = fn() { a };
            let second = fn() { b };
            fn() { first() + second() }
        };
        let closure = new_closure(9, 90);
        closure();",
    ];
    let expected = [
        Object::Integer(9),
        Object::Integer(4),
        Object::Integer(11),
        Object::Integer(11),
        Object::Integer(14),
        Object::Integer(14),
        Object::Integer(99),
    ];
    assert_vm_runs(&input, &expected);
}

#[test]
fn test_recursive_fibonacci() {
    let input = ["
        let fibonacci = fn(n) {
            if n < 2 {
                n
            } else {
                fibonacci(n - 1) + fibonacci(n - 2)
            }
        };
        fibonacci(30)
    "];
    let expected = [Object::Integer(610)];
    assert_vm_runs(&input, &expected);
}
