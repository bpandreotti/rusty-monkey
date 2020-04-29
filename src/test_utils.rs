#![cfg(test)]
use crate::ast::*;
use crate::code::*;
use crate::compiler::*;
use crate::environment::*;
use crate::error::*;
use crate::eval::*;
use crate::lexer::*;
use crate::object::*;
use crate::parser::*;
use crate::token::*;
use crate::vm::*;

use std::rc::Rc;
use std::cell::RefCell;

pub fn parse(program: &str) -> Result<Vec<NodeStatement>, MonkeyError> {
    let lex = Lexer::from_string(program.into())?;
    let mut parser = Parser::new(lex)?;
    parser.parse_program()
}

pub fn parse_and_compile(program: &str) -> Result<Bytecode, MonkeyError> {
    let parsed = parse(program)?;
    let mut comp = Compiler::new();
    comp.compile_program(parsed)?;
    Ok(comp.bytecode())
}

pub fn assert_object_integer(expected: i64, got: &Object) {
    match got {
        Object::Integer(i) => assert_eq!(expected, *i),
        _ => panic!("Wrong object type"),
    }
}

pub fn assert_object_bool(expected: bool, got: &Object) {
    match got {
        Object::Boolean(p) => assert_eq!(expected, *p),
        _ => panic!("Wrong object type"),
    }
}

pub fn assert_lex(input: &str, expected: &[Token]) {
    let mut lex = Lexer::from_string(input.into()).expect("Lexer error during test");
    for ex in expected {
        let got = lex.next_token().expect("Lexer error during test");
        assert_eq!(ex, &got);
    }
}

pub fn assert_lexer_error(input: &str, expected_error: LexerError) {
    let mut lex = Lexer::from_string(input.into()).unwrap();
    loop {
        match lex.next_token() {
            Ok(Token::EOF) => panic!("No lexer errors encountered"),
            Err(e) => {
                match e.error {
                    ErrorType::Lexer(got) => assert_eq!(expected_error, got),
                    _ => panic!("Wrong error type")
                }
                return;
            }
            _ => continue,
        }
    }
}

pub fn assert_parse(input: &str, expected: &[&str]) {
    let output = parse(input).expect("Parser error during test");
    assert_eq!(output.len(), expected.len());

    for i in 0..output.len() {
        assert_eq!(format!("{:?}", output[i]), expected[i]);
    }
}

pub fn assert_parse_fails(input: &str) {
    assert!(parse(input).is_err());
}


pub fn assert_eval(input: &str, expected: &[Object]) {
    let parsed = parse(input).expect("Parser error during test");
    assert_eq!(parsed.len(), expected.len());
    let env = Rc::new(RefCell::new(Environment::empty()));

    // Eval program statements and compare with expected
    for (statement, exp) in parsed.into_iter().zip(expected) {
        let got = eval_statement(&statement, &env).expect("Runtime error during test");
        assert_eq!(format!("{}", got), format!("{}", exp));
    }
}

pub fn assert_runtime_error(input: &str, expected_errors: &[&str]) {
    let parsed = parse(input).expect("Parser error during test");
    let env = Rc::new(RefCell::new(Environment::empty()));
    for (statement, &error) in parsed.iter().zip(expected_errors) {
        let got = eval_statement(statement, &env).expect_err("No runtime error encountered");
        match got.error {
            ErrorType::Runtime(e) => assert_eq!(e.message(), error),
            _ => panic!("Wrong error type"),
        }
    }
}

// @TODO: Also compare constants
pub fn assert_compile(input: &str, expected: Instructions) {
    let program = parse(input).expect("Parser error during test");
    let mut comp = Compiler::new();
    comp.compile_program(program).expect("Compiler error during test");
    assert_eq!(expected, comp.bytecode().instructions)
}

pub fn assert_vm_runs(input: &[&str], expected: &[Object]) {
    for (program, exp) in input.iter().zip(expected) {
        let bytecode = parse_and_compile(program).expect("Parser or compiler error during test");
        let mut vm = VM::new(bytecode);
        vm.run().unwrap();
        assert!(Object::are_equal(exp, vm.last_popped()).unwrap());
    }
}
