use crate::environment::*;
use crate::error;
use crate::eval;
use crate::lexer::Lexer;
use crate::object;
use crate::parser::Parser;

use std::io::BufRead;
use std::rc::Rc;
use std::cell::RefCell;

const PROMPT: &str = "monkey » ";

pub fn start() -> Result<(), std::io::Error> {
    eprintln!("Hello! This is the Monkey programming language!");

    let stdin = std::io::stdin();
    let env = Rc::new(RefCell::new(Environment::empty()));
    eprint!("{}", PROMPT);
    for line in stdin.lock().lines() {
        let line = line?;
        if line == "exit" {
            break;
        }

        match run_line(line, &env) {
            Ok(values) => for v in values {
                println!("{}", v);
            }
            Err(e) => eprintln!("{}", e),
        }
        eprint!("{}", PROMPT);
    }

    eprintln!("Goodbye!");
    Ok(())
}

fn run_line(line: String, env: &EnvHandle) -> Result<Vec<object::Object>, error::MonkeyError> {
    let lexer = Lexer::from_string(line)?;
    let mut parser = Parser::new(lexer)?;
    let mut results = Vec::new();
    for statement in parser.parse_program()? {
        results.push(eval::eval_statement(&statement, env)?);
    }
    Ok(results)
}
