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
    ];
    let expected = [Object::Integer(15), Object::Integer(15)];
    assert_vm_runs(&input, &expected);
}

// The way return statements currently work in the VM, they just pop the current Frame off of the
// frame stack, but leave the objects stack untouched. This means that if a function returns while
//  it's using the stack to execute an operation, those values will remain on the stack.
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
