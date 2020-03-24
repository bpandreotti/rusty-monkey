use crate::eval;
use crate::lexer::Lexer;
use crate::parser::Parser;

use std::io::BufRead;

const PROMPT: &str = "monkey » ";

pub fn start() -> Result<(), std::io::Error> {
    println!("Hello! This is the Monkey programming language!");

    let stdin = std::io::stdin();
    eprint!("{}", PROMPT);
    for line in stdin.lock().lines() {
        let line = line?;
        if line == "exit" {
            break;
        }

        let program = Parser::new(Lexer::new(line)).parse_program();

        match program {
            Ok(statements) => {
                for s in statements {
                    println!("{}", eval::eval_statement(s));
                }
            }
            Err(e) => println!("PARSER ERROR: {:?}", e),
        }
        eprint!("{}", PROMPT);
    }

    println!("Goodbye!");
    Ok(())
}
