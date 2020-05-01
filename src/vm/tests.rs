use super::*;
use crate::test_utils;

fn assert_vm_runs(input: &[&str], expected: &[Object]) {
    for (program, exp) in input.iter().zip(expected) {
        let bytecode =
            test_utils::parse_and_compile(program).expect("Parser or compiler error during test");
        let mut vm = VM::new(bytecode);
        vm.run().unwrap();
        assert!(test_utils::compare_objects(exp, vm.stack_top().unwrap()));
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
