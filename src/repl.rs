use std::io::BufRead;

use crate::lexer::Lexer;
use crate::token::Token;

const PROMPT: &str = "monkey » ";

pub fn start() -> Result<(), std::io::Error> {
    let stdin = std::io::stdin();
    eprint!("{}", PROMPT);
    for line in stdin.lock().lines() {
        let line = line?;
        if line == "exit" {
            break;
        }

        let mut lex = Lexer::new(line);
        let mut tk = lex.next_token();
        while tk != Token::EOF {
            println!("{:?}", tk);
            tk = lex.next_token();
        }
        eprint!("{}", PROMPT);
    }

    println!("Goodbye!");
    Ok(())
}
