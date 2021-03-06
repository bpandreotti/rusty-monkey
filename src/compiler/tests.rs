use super::*;
use crate::test_utils;

macro_rules! instructions {
    ($( ($( $operands:expr ),* ) ),* $(,)?) => {
        Instructions(
            [ $( make!( $( $operands ),* ) ),* ].concat()
        )
    };
}

fn assert_compile(
    input: &str,
    expected_constants: Vec<Object>,
    expected_instructions: code::Instructions,
) {
    let bytecode =
        test_utils::parse_and_compile(input).expect("Parser or compiler error during test");
    for (exp, got) in expected_constants.iter().zip(bytecode.constants) {
        assert_eq!(exp, &got);
    }
    assert_eq!(expected_instructions, bytecode.instructions);
}

#[test]
fn test_make() {
    assert_eq!(
        &[OpCode::OpConstant as u8, 255, 254],
        &*make!(OpCode::OpConstant, 65534)
    );
    assert_eq!(
        &[OpCode::OpGetLocal as u8, 255],
        &*make!(OpCode::OpGetLocal, 255)
    );
    assert_eq!(&[OpCode::OpAdd as u8], &*make!(OpCode::OpAdd));
    assert_eq!(
        &[OpCode::OpClosure as u8, 255, 254, 42],
        &*make!(OpCode::OpClosure, 65534, 42)
    );
}

#[test]
fn test_instruction_printing() {
    let input = instructions! {
        (OpCode::OpAdd),
        (OpCode::OpConstant, 2),
        (OpCode::OpConstant, 65535),
        (OpCode::OpClosure, 65534, 42),
    };
    let expected = "\
    0000 OpAdd\n\
    0001 OpConstant 2\n\
    0004 OpConstant 65535\n\
    0007 OpClosure 65534 42\n\
    ";
    assert_eq!(expected, format!("{}", input));
}

#[test]
fn test_integer_arithmetic() {
    assert_compile(
        "1 + 2",
        vec![Object::Integer(1), Object::Integer(2)],
        instructions! {
            (OpCode::OpConstant, 0),
            (OpCode::OpConstant, 1),
            (OpCode::OpAdd),
        },
    );
    assert_compile(
        "1; 2",
        vec![Object::Integer(1), Object::Integer(2)],
        instructions! {
            (OpCode::OpConstant, 0),
            (OpCode::OpPop),
            (OpCode::OpConstant, 1),
        },
    );
    assert_compile(
        "1 * 2",
        vec![Object::Integer(1), Object::Integer(2)],
        instructions! {
            (OpCode::OpConstant, 0),
            (OpCode::OpConstant, 1),
            (OpCode::OpMul),
        },
    );
    assert_compile(
        "-1",
        vec![Object::Integer(1)],
        instructions! {
            (OpCode::OpConstant, 0),
            (OpCode::OpPrefixMinus)
        },
    );
}

#[test]
fn test_boolean_expressions() {
    assert_compile("true", vec![], instructions! { (OpCode::OpTrue) });
    assert_compile("false", vec![], instructions! { (OpCode::OpFalse) });
    assert_compile(
        "1 > 2",
        vec![Object::Integer(1), Object::Integer(2)],
        instructions! {
            (OpCode::OpConstant, 0),
            (OpCode::OpConstant, 1),
            (OpCode::OpGreaterThan),
        },
    );
    assert_compile(
        "1 < 2",
        vec![Object::Integer(2), Object::Integer(1)],
        instructions! {
            (OpCode::OpConstant, 0),
            (OpCode::OpConstant, 1),
            (OpCode::OpGreaterThan),
        },
    );
    assert_compile(
        "1 == 2",
        vec![Object::Integer(1), Object::Integer(2)],
        instructions! {
            (OpCode::OpConstant, 0),
            (OpCode::OpConstant, 1),
            (OpCode::OpEquals),
        },
    );
    assert_compile(
        "1 != 2",
        vec![Object::Integer(1), Object::Integer(2)],
        instructions! {
            (OpCode::OpConstant, 0),
            (OpCode::OpConstant, 1),
            (OpCode::OpNotEquals),
        },
    );
    assert_compile(
        "!true",
        vec![],
        instructions! { (OpCode::OpTrue), (OpCode::OpPrefixNot) },
    );
}

#[test]
fn test_conditionals() {
    assert_compile(
        "if true { 10 }; 3333",
        vec![Object::Integer(10), Object::Integer(3333)],
        instructions! {
            (OpCode::OpTrue),
            (OpCode::OpJumpNotTruthy, 10),
            (OpCode::OpConstant, 0),
            (OpCode::OpJump, 11),
            (OpCode::OpNil),
            (OpCode::OpPop),
            (OpCode::OpConstant, 1),
        },
    );
    assert_compile(
        "if true { 10 } else { 20 }; 3333",
        vec![
            Object::Integer(10),
            Object::Integer(20),
            Object::Integer(3333),
        ],
        instructions! {
            (OpCode::OpTrue),
            (OpCode::OpJumpNotTruthy, 10),
            (OpCode::OpConstant, 0),
            (OpCode::OpJump, 13),
            (OpCode::OpConstant, 1),
            (OpCode::OpPop),
            (OpCode::OpConstant, 2),
        },
    );
}

#[test]
fn test_global_assignment() {
    assert_compile(
        "let one = 1; let two = 2",
        vec![Object::Integer(1), Object::Integer(2)],
        instructions! {
            (OpCode::OpConstant, 0),
            (OpCode::OpSetGlobal, 0),
            (OpCode::OpConstant, 1),
            (OpCode::OpSetGlobal, 1),
            (OpCode::OpNil),
        },
    );
    assert_compile(
        "let one = 1; one",
        vec![Object::Integer(1)],
        instructions! {
            (OpCode::OpConstant, 0),
            (OpCode::OpSetGlobal, 0),
            (OpCode::OpGetGlobal, 0),
        },
    );
    assert_compile(
        "let one = 1; let two = one; two",
        vec![Object::Integer(1)],
        instructions! {
            (OpCode::OpConstant, 0),
            (OpCode::OpSetGlobal, 0),
            (OpCode::OpGetGlobal, 0),
            (OpCode::OpSetGlobal, 1),
            (OpCode::OpGetGlobal, 1),
        },
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
        instructions! {
            (OpCode::OpConstant, 0),
            (OpCode::OpConstant, 1),
            (OpCode::OpAdd),
        },
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
        instructions! {
            (OpCode::OpConstant, 0),
            (OpCode::OpConstant, 1),
            (OpCode::OpConstant, 2),
            (OpCode::OpArray, 3),
        },
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
        instructions! {
            (OpCode::OpConstant, 0),
            (OpCode::OpConstant, 1),
            (OpCode::OpAdd),
            (OpCode::OpConstant, 2),
            (OpCode::OpConstant, 3),
            (OpCode::OpSub),
            (OpCode::OpConstant, 4),
            (OpCode::OpConstant, 5),
            (OpCode::OpMul),
            (OpCode::OpArray, 3),
        },
    );
}

#[test]
fn test_hashes() {
    assert_compile("#{}", vec![], instructions! { (OpCode::OpHash, 0) });
    assert_compile(
        "#{ 1: 2, 3: 4 }",
        vec![
            Object::Integer(1),
            Object::Integer(2),
            Object::Integer(3),
            Object::Integer(4),
        ],
        instructions! {
            (OpCode::OpConstant, 0),
            (OpCode::OpConstant, 1),
            (OpCode::OpConstant, 2),
            (OpCode::OpConstant, 3),
            (OpCode::OpHash, 2),
        },
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
        instructions! {
            (OpCode::OpConstant, 0),
            (OpCode::OpConstant, 1),
            (OpCode::OpConstant, 2),
            (OpCode::OpAdd),
            (OpCode::OpConstant, 3),
            (OpCode::OpConstant, 4),
            (OpCode::OpConstant, 5),
            (OpCode::OpMul),
            (OpCode::OpHash, 2),
        },
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
        instructions! {
            (OpCode::OpConstant, 0),
            (OpCode::OpConstant, 1),
            (OpCode::OpConstant, 2),
            (OpCode::OpArray, 3),
            (OpCode::OpConstant, 3),
            (OpCode::OpConstant, 4),
            (OpCode::OpAdd),
            (OpCode::OpIndex),
        },
    );
    assert_compile(
        "#{ 1: 2 }[2 - 1]",
        vec![
            Object::Integer(1),
            Object::Integer(2),
            Object::Integer(2),
            Object::Integer(1),
        ],
        instructions! {
            (OpCode::OpConstant, 0),
            (OpCode::OpConstant, 1),
            (OpCode::OpHash, 1),
            (OpCode::OpConstant, 2),
            (OpCode::OpConstant, 3),
            (OpCode::OpSub),
            (OpCode::OpIndex),
        },
    );
}

#[test]
fn test_function_literals() {
    let expected_func = Object::CompiledFunc(Box::new(CompiledFunction {
        instructions: instructions! {
            (OpCode::OpConstant, 0),
            (OpCode::OpConstant, 1),
            (OpCode::OpAdd),
            (OpCode::OpReturn),
        },
        num_locals: 0,
        num_params: 0,
    }));
    assert_compile(
        "fn() { return 5 + 10; }",
        vec![Object::Integer(5), Object::Integer(10), expected_func],
        instructions! { (OpCode::OpClosure, 2, 0) },
    );
    let expected_func = Object::CompiledFunc(Box::new(CompiledFunction {
        instructions: instructions! {
            (OpCode::OpConstant, 0),
            (OpCode::OpReturn),
        },
        num_locals: 0,
        num_params: 0,
    }));
    assert_compile(
        "fn() { 1 }",
        vec![Object::Integer(1), expected_func],
        instructions! { (OpCode::OpClosure, 1, 0) },
    );
}

#[test]
fn test_function_calls() {
    let expected_func = Object::CompiledFunc(Box::new(CompiledFunction {
        instructions: instructions! {
            (OpCode::OpConstant, 0),
            (OpCode::OpReturn),
        },
        num_locals: 0,
        num_params: 0,
    }));
    assert_compile(
        "fn() { 24 }()",
        vec![Object::Integer(24), expected_func.clone()],
        instructions! { (OpCode::OpClosure, 1, 0), (OpCode::OpCall, 0) },
    );
    assert_compile(
        "let foo = fn() { 24 }; foo()",
        vec![Object::Integer(24), expected_func],
        instructions! {
            (OpCode::OpClosure, 1, 0),
            (OpCode::OpSetGlobal, 0),
            (OpCode::OpGetGlobal, 0),
            (OpCode::OpCall, 0),
        },
    );

    let expected_func = Object::CompiledFunc(Box::new(CompiledFunction {
        instructions: instructions! { (OpCode::OpGetLocal, 0), (OpCode::OpReturn) },
        num_locals: 1,
        num_params: 1,
    }));
    assert_compile(
        "let one_arg = fn(x) { x }; one_arg(0)",
        vec![expected_func, Object::Integer(0)],
        instructions! {
            (OpCode::OpClosure, 0, 0),
            (OpCode::OpSetGlobal, 0),
            (OpCode::OpGetGlobal, 0),
            (OpCode::OpConstant, 1),
            (OpCode::OpCall, 1),
        },
    );

    let expected_func = Object::CompiledFunc(Box::new(CompiledFunction {
        instructions: instructions! {
           (OpCode::OpGetLocal, 0),
           (OpCode::OpPop),
           (OpCode::OpGetLocal, 1),
           (OpCode::OpPop),
           (OpCode::OpGetLocal, 2),
           (OpCode::OpReturn),
        },
        num_locals: 3,
        num_params: 3,
    }));
    assert_compile(
        "let many_arg = fn(x, y, z) { x; y; z }; many_arg(24, 25, 26)",
        vec![
            expected_func,
            Object::Integer(24),
            Object::Integer(25),
            Object::Integer(26),
        ],
        instructions! {
            (OpCode::OpClosure, 0, 0),
            (OpCode::OpSetGlobal, 0),
            (OpCode::OpGetGlobal, 0),
            (OpCode::OpConstant, 1),
            (OpCode::OpConstant, 2),
            (OpCode::OpConstant, 3),
            (OpCode::OpCall, 3),
        },
    );
}

#[test]
fn test_binding_scopes() {
    let expected_func = Object::CompiledFunc(Box::new(CompiledFunction {
        instructions: instructions! {
            (OpCode::OpGetGlobal, 0),
            (OpCode::OpReturn),
        },
        num_locals: 0,
        num_params: 0,
    }));
    assert_compile(
        "let num = 55; fn() { num }",
        vec![Object::Integer(55), expected_func],
        instructions! {
            (OpCode::OpConstant, 0),
            (OpCode::OpSetGlobal, 0),
            (OpCode::OpClosure, 1, 0),
        },
    );
    let expected_func = Object::CompiledFunc(Box::new(CompiledFunction {
        instructions: instructions! {
            (OpCode::OpConstant, 0),
            (OpCode::OpSetLocal, 0),
            (OpCode::OpGetLocal, 0),
            (OpCode::OpReturn),
        },
        num_locals: 1,
        num_params: 0,
    }));
    assert_compile(
        "fn() { let num = 55; num }",
        vec![Object::Integer(55), expected_func],
        instructions! { (OpCode::OpClosure, 1, 0) },
    );
}

#[test]
fn test_builtins() {
    assert_compile(
        "len([]); push([], 1);",
        vec![Object::Integer(1)],
        instructions! {
            (OpCode::OpGetBuiltin, 2),
            (OpCode::OpArray, 0),
            (OpCode::OpCall, 1),
            (OpCode::OpPop),
            (OpCode::OpGetBuiltin, 3),
            (OpCode::OpArray, 0),
            (OpCode::OpConstant, 0),
            (OpCode::OpCall, 2),
        },
    );
    let expected_func = Object::CompiledFunc(Box::new(CompiledFunction {
        instructions: instructions! {
            (OpCode::OpGetBuiltin, 2),
            (OpCode::OpArray, 0),
            (OpCode::OpCall, 1),
            (OpCode::OpReturn),
        },
        num_locals: 0,
        num_params: 0,
    }));
    assert_compile(
        "fn() { len([]) }",
        vec![expected_func],
        instructions! {
            (OpCode::OpClosure, 0, 0),
        },
    );
}

#[test]
fn test_closures() {
    let outer_func = Object::CompiledFunc(Box::new(CompiledFunction {
        instructions: instructions! {
            (OpCode::OpGetLocal, 0),
            (OpCode::OpClosure, 0, 1),
            (OpCode::OpReturn),
        },
        num_locals: 1,
        num_params: 1,
    }));
    let inner_func = Object::CompiledFunc(Box::new(CompiledFunction {
        instructions: instructions! {
            (OpCode::OpGetFree, 0),
            (OpCode::OpGetLocal, 0),
            (OpCode::OpAdd),
            (OpCode::OpReturn),
        },
        num_locals: 1,
        num_params: 1,
    }));
    assert_compile(
        "fn(a) { fn(b) { a + b } }",
        vec![inner_func, outer_func],
        instructions! { (OpCode::OpClosure, 1, 0) },
    );

    let outer_func = Object::CompiledFunc(Box::new(CompiledFunction {
        instructions: instructions! {
            (OpCode::OpGetLocal, 0),
            (OpCode::OpClosure, 1, 1),
            (OpCode::OpReturn),
        },
        num_locals: 1,
        num_params: 1,
    }));
    let inner_func = Object::CompiledFunc(Box::new(CompiledFunction {
        instructions: instructions! {
            (OpCode::OpGetFree, 0),
            (OpCode::OpGetLocal, 0),
            (OpCode::OpClosure, 0, 2),
            (OpCode::OpReturn),
        },
        num_locals: 1,
        num_params: 1,
    }));
    let inner_inner_func = Object::CompiledFunc(Box::new(CompiledFunction {
        instructions: instructions! {
            (OpCode::OpGetFree, 0),
            (OpCode::OpGetFree, 1),
            (OpCode::OpAdd),
            (OpCode::OpGetLocal, 0),
            (OpCode::OpAdd),
            (OpCode::OpReturn),
        },
        num_locals: 1,
        num_params: 1,
    }));
    assert_compile(
        "fn(a) { fn(b) { fn(c) { a + b + c } } }",
        vec![inner_inner_func, inner_func, outer_func],
        instructions! { (OpCode::OpClosure, 2, 0) },
    );
}
