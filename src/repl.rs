use std::io::BufRead;

use crate::lexer::Lexer;
use crate::parser::Parser;

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

        let lex = Lexer::new(line);
        let mut pars = Parser::new(lex);

        match pars.parse_program() {
            Ok(statements) => for s in statements {
                println!("{:#?}", s);
            },
            Err(e) => println!("PARSER ERROR: {:?}", e),
        }

        eprint!("{}", PROMPT);
    }

    println!("Goodbye!");
    Ok(())
}
