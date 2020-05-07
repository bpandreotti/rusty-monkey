use super::*;
use crate::test_utils;

fn assert_compile(
    input: &str,
    expected_constants: Vec<Object>,
    expected_instructions: code::Instructions,
) {
    let bytecode =
        test_utils::parse_and_compile(input).expect("Parser or compiler error during test");
    for (exp, got) in expected_constants.iter().zip(bytecode.constants) {
        assert!(test_utils::compare_vm_objects(exp, &got));
    }
    assert_eq!(expected_instructions, bytecode.instructions);
}

#[test]
fn test_make() {
    assert_eq!(
        &[OpCode::OpConstant as u8, 255, 254],
        &*make!(OpCode::OpConstant, 65534)
    );
    assert_eq!(&[OpCode::OpAdd as u8], &*make!(OpCode::OpAdd));
}

#[test]
fn test_instruction_printing() {
    let input = Instructions(
        [
            make!(OpCode::OpAdd),
            make!(OpCode::OpConstant, 2),
            make!(OpCode::OpConstant, 65535),
        ]
        .concat(),
    );
    let expected = "\
    0000 OpAdd\n\
    0001 OpConstant 2\n\
    0004 OpConstant 65535\n\
    ";
    assert_eq!(expected, format!("{}", input));
}

#[test]
fn test_integer_arithmetic() {
    assert_compile(
        "1 + 2",
        vec![Object::Integer(1), Object::Integer(2)],
        Instructions(
            [
                make!(OpCode::OpConstant, 0),
                make!(OpCode::OpConstant, 1),
                make!(OpCode::OpAdd),
            ]
            .concat(),
        ),
    );
    assert_compile(
        "1; 2",
        vec![Object::Integer(1), Object::Integer(2)],
        Instructions(
            [
                make!(OpCode::OpConstant, 0),
                make!(OpCode::OpPop),
                make!(OpCode::OpConstant, 1),
            ]
            .concat(),
        ),
    );
    assert_compile(
        "1 * 2",
        vec![Object::Integer(1), Object::Integer(2)],
        Instructions(
            [
                make!(OpCode::OpConstant, 0),
                make!(OpCode::OpConstant, 1),
                make!(OpCode::OpMul),
            ]
            .concat(),
        ),
    );
    assert_compile(
        "-1",
        vec![Object::Integer(1)],
        Instructions([make!(OpCode::OpConstant, 0), make!(OpCode::OpPrefixMinus)].concat()),
    );
}

#[test]
fn test_boolean_expressions() {
    assert_compile(
        "true",
        vec![],
        Instructions([make!(OpCode::OpTrue)].concat()),
    );
    assert_compile(
        "false",
        vec![],
        Instructions([make!(OpCode::OpFalse)].concat()),
    );
    assert_compile(
        "1 > 2",
        vec![Object::Integer(1), Object::Integer(2)],
        Instructions(
            [
                make!(OpCode::OpConstant, 0),
                make!(OpCode::OpConstant, 1),
                make!(OpCode::OpGreaterThan),
            ]
            .concat(),
        ),
    );
    assert_compile(
        "1 < 2",
        vec![Object::Integer(2), Object::Integer(1)],
        Instructions(
            [
                make!(OpCode::OpConstant, 0),
                make!(OpCode::OpConstant, 1),
                make!(OpCode::OpGreaterThan),
            ]
            .concat(),
        ),
    );
    assert_compile(
        "1 == 2",
        vec![Object::Integer(1), Object::Integer(2)],
        Instructions(
            [
                make!(OpCode::OpConstant, 0),
                make!(OpCode::OpConstant, 1),
                make!(OpCode::OpEquals),
            ]
            .concat(),
        ),
    );
    assert_compile(
        "1 != 2",
        vec![Object::Integer(1), Object::Integer(2)],
        Instructions(
            [
                make!(OpCode::OpConstant, 0),
                make!(OpCode::OpConstant, 1),
                make!(OpCode::OpNotEquals),
            ]
            .concat(),
        ),
    );
    assert_compile(
        "!true",
        vec![],
        Instructions([make!(OpCode::OpTrue), make!(OpCode::OpPrefixNot)].concat()),
    );
}

#[test]
fn test_conditionals() {
    assert_compile(
        "if true { 10 }; 3333",
        vec![Object::Integer(10), Object::Integer(3333)],
        Instructions(
            [
                make!(OpCode::OpTrue),
                make!(OpCode::OpJumpNotTruthy, 10),
                make!(OpCode::OpConstant, 0),
                make!(OpCode::OpJump, 11),
                make!(OpCode::OpNil),
                make!(OpCode::OpPop),
                make!(OpCode::OpConstant, 1),
            ]
            .concat(),
        ),
    );
    assert_compile(
        "if true { 10 } else { 20 }; 3333",
        vec![
            Object::Integer(10),
            Object::Integer(20),
            Object::Integer(3333),
        ],
        Instructions(
            [
                make!(OpCode::OpTrue),
                make!(OpCode::OpJumpNotTruthy, 10),
                make!(OpCode::OpConstant, 0),
                make!(OpCode::OpJump, 13),
                make!(OpCode::OpConstant, 1),
                make!(OpCode::OpPop),
                make!(OpCode::OpConstant, 2),
            ]
            .concat(),
        ),
    );
}

#[test]
fn test_global_assignment() {
    assert_compile(
        "let one = 1; let two = 2",
        vec![Object::Integer(1), Object::Integer(2)],
        Instructions(
            [
                make!(OpCode::OpConstant, 0),
                make!(OpCode::OpSetGlobal, 0),
                make!(OpCode::OpConstant, 1),
                make!(OpCode::OpSetGlobal, 1),
                make!(OpCode::OpNil),
            ]
            .concat(),
        ),
    );
    assert_compile(
        "let one = 1; one",
        vec![Object::Integer(1)],
        Instructions(
            [
                make!(OpCode::OpConstant, 0),
                make!(OpCode::OpSetGlobal, 0),
                make!(OpCode::OpGetGlobal, 0),
            ]
            .concat(),
        ),
    );
    assert_compile(
        "let one = 1; let two = one; two",
        vec![Object::Integer(1)],
        Instructions(
            [
                make!(OpCode::OpConstant, 0),
                make!(OpCode::OpSetGlobal, 0),
                make!(OpCode::OpGetGlobal, 0),
                make!(OpCode::OpSetGlobal, 1),
                make!(OpCode::OpGetGlobal, 1),
            ]
            .concat(),
        ),
    );
}

#[test]
fn test_strings() {
    assert_compile(
        "\"monkey\"",
        vec![],
        Instructions([make!(OpCode::OpConstant, 0)].concat()),
    );
    assert_compile(
        "\"mon\" + \"key\"",
        vec![],
        Instructions(
            [
                make!(OpCode::OpConstant, 0),
                make!(OpCode::OpConstant, 1),
                make!(OpCode::OpAdd),
            ]
            .concat(),
        ),
    );
}

#[test]
fn test_arrays() {
    assert_compile(
        "[]",
        vec![],
        Instructions([make!(OpCode::OpArray, 0)].concat()),
    );
    assert_compile(
        "[1, 2, 3]",
        vec![Object::Integer(1), Object::Integer(2), Object::Integer(3)],
        Instructions(
            [
                make!(OpCode::OpConstant, 0),
                make!(OpCode::OpConstant, 1),
                make!(OpCode::OpConstant, 2),
                make!(OpCode::OpArray, 3),
            ]
            .concat(),
        ),
    );
    assert_compile(
        "[1 + 2, 3 - 4, 5 * 6]",
        vec![
            Object::Integer(1),
            Object::Integer(2),
            Object::Integer(3),
            Object::Integer(4),
            Object::Integer(5),
            Object::Integer(6),
        ],
        Instructions(
            [
                make!(OpCode::OpConstant, 0),
                make!(OpCode::OpConstant, 1),
                make!(OpCode::OpAdd),
                make!(OpCode::OpConstant, 2),
                make!(OpCode::OpConstant, 3),
                make!(OpCode::OpSub),
                make!(OpCode::OpConstant, 4),
                make!(OpCode::OpConstant, 5),
                make!(OpCode::OpMul),
                make!(OpCode::OpArray, 3),
            ]
            .concat(),
        ),
    );
}

#[test]
fn test_hashes() {
    assert_compile(
        "#{}",
        vec![],
        Instructions([make!(OpCode::OpHash, 0)].concat()),
    );
    assert_compile(
        "#{ 1: 2, 3: 4 }",
        vec![
            Object::Integer(1),
            Object::Integer(2),
            Object::Integer(3),
            Object::Integer(4),
        ],
        Instructions(
            [
                make!(OpCode::OpConstant, 0),
                make!(OpCode::OpConstant, 1),
                make!(OpCode::OpConstant, 2),
                make!(OpCode::OpConstant, 3),
                make!(OpCode::OpHash, 2),
            ]
            .concat(),
        ),
    );
    assert_compile(
        "#{ 1: 2 + 3, 4: 5 * 6 }",
        vec![
            Object::Integer(1),
            Object::Integer(2),
            Object::Integer(3),
            Object::Integer(4),
            Object::Integer(5),
            Object::Integer(6),
        ],
        Instructions(
            [
                make!(OpCode::OpConstant, 0),
                make!(OpCode::OpConstant, 1),
                make!(OpCode::OpConstant, 2),
                make!(OpCode::OpAdd),
                make!(OpCode::OpConstant, 3),
                make!(OpCode::OpConstant, 4),
                make!(OpCode::OpConstant, 5),
                make!(OpCode::OpMul),
                make!(OpCode::OpHash, 2),
            ]
            .concat(),
        ),
    );
}

#[test]
fn test_index_expressions() {
    assert_compile(
        "[1, 2, 3][1 + 1]",
        vec![
            Object::Integer(1),
            Object::Integer(2),
            Object::Integer(3),
            Object::Integer(1),
            Object::Integer(1),
        ],
        Instructions(
            [
                make!(OpCode::OpConstant, 0),
                make!(OpCode::OpConstant, 1),
                make!(OpCode::OpConstant, 2),
                make!(OpCode::OpArray, 3),
                make!(OpCode::OpConstant, 3),
                make!(OpCode::OpConstant, 4),
                make!(OpCode::OpAdd),
                make!(OpCode::OpIndex),
            ]
            .concat(),
        ),
    );
    assert_compile(
        "#{ 1: 2 }[2 - 1]",
        vec![
            Object::Integer(1),
            Object::Integer(2),
            Object::Integer(2),
            Object::Integer(1),
        ],
        Instructions(
            [
                make!(OpCode::OpConstant, 0),
                make!(OpCode::OpConstant, 1),
                make!(OpCode::OpHash, 1),
                make!(OpCode::OpConstant, 2),
                make!(OpCode::OpConstant, 3),
                make!(OpCode::OpSub),
                make!(OpCode::OpIndex),
            ]
            .concat(),
        ),
    );
}

#[test]
fn test_function_literals() {
    let expected_func = Object::CompiledFunction(Instructions(
        [
            make!(OpCode::OpConstant, 0),
            make!(OpCode::OpConstant, 1),
            make!(OpCode::OpAdd),
            make!(OpCode::OpReturn),
        ]
        .concat(),
    ));
    assert_compile(
        "fn() { return 5 + 10; }",
        vec![Object::Integer(5), Object::Integer(10), expected_func],
        Instructions(make!(OpCode::OpConstant, 2).into()),
    );
    let expected_func = Object::CompiledFunction(Instructions(make!(OpCode::OpConstant, 0).into()));
    assert_compile(
        "fn() { 1 }",
        vec![Object::Integer(1), expected_func],
        Instructions(make!(OpCode::OpConstant, 1).into()),
    );
}

#[test]
fn test_function_calls() {
    let expected_func = Object::CompiledFunction(Instructions(make!(OpCode::OpConstant, 0).into()));
    assert_compile(
        "fn() { 24 }()",
        vec![Object::Integer(24), expected_func.clone()],
        Instructions([make!(OpCode::OpConstant, 1), make!(OpCode::OpCall)].concat()),
    );
    assert_compile(
        "let foo = fn() { 24 }; foo()",
        vec![Object::Integer(24), expected_func],
        Instructions(
            [
                make!(OpCode::OpConstant, 1),
                make!(OpCode::OpSetGlobal, 0),
                make!(OpCode::OpGetGlobal, 0),
                make!(OpCode::OpCall),
            ]
            .concat(),
        ),
    );
}
