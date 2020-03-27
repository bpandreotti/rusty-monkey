use crate::environment::Environment;
use crate::eval;
use crate::lexer::Lexer;
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

        let program = Parser::new(Lexer::new(line)).parse_program();
        
        match program {
            Ok(statements) => statements
                .into_iter()
                .for_each(|s| match eval::eval_statement(&s, &env) {
                    Ok(obj) => println!("{}", obj),
                    Err(e) => eprintln!("Runtime Error: {}", e),
                }),
            Err(e) => eprintln!("Parser Error: {}", e),
        }
        eprint!("{}", PROMPT);
    }

    eprintln!("Goodbye!");
    Ok(())
}
