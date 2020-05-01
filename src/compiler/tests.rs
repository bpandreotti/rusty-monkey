use super::*;
use crate::test_utils;

// @TODO: Also compare constants
fn assert_compile(input: &str, expected: code::Instructions) {
    let bytecode =
        test_utils::parse_and_compile(input).expect("Parser or compiler error during test");
    assert_eq!(expected, bytecode.instructions)
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
        Instructions([make!(OpCode::OpConstant, 0), make!(OpCode::OpPrefixMinus)].concat()),
    );
}

#[test]
fn test_boolean_expressions() {
    assert_compile("true", Instructions([make!(OpCode::OpTrue)].concat()));
    assert_compile("false", Instructions([make!(OpCode::OpFalse)].concat()));
    assert_compile(
        "1 > 2",
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
        Instructions([make!(OpCode::OpTrue), make!(OpCode::OpPrefixNot)].concat()),
    );
}

#[test]
fn test_conditionals() {
    assert_compile(
        "if true { 10 }; 3333",
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
