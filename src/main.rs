mod ast;
mod builtins;
mod environment;
mod eval;
mod lexer;
mod object;
mod parser;
mod repl;
mod token;
mod error;

use std::error::Error;
use std::fs;


fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args();
    if let Some(path) = args.nth(1) {
        run_program_file(path)?;
    } else {
        repl::start()?;
    }
    Ok(())
}

fn run_program_file(path: String) -> Result<(), Box<dyn Error>> {
    // @TODO: Read file using BufRead
    let contents = fs::read_to_string(path)?;
    let lexer = lexer::Lexer::from_string(contents);
    let parsed_program = parser::Parser::new(lexer).parse_program()?;
    eval::run_program(parsed_program)?;
    Ok(())
}
