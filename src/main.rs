mod compiler;
mod error;
mod interpreter;
mod lexer;
mod parser;
mod repl;
#[cfg(test)]
mod test_utils;
mod vm;

use error::MonkeyError;
use std::fs::File;
use std::io::BufReader;

fn main() -> Result<(), MonkeyError> {
    let mut args = std::env::args();
    match args.nth(1).as_deref() {
        Some("-c") | Some("c") | None => repl::start(true).unwrap(),
        Some("-i") | Some("i") => repl::start(false).unwrap(),
        Some(path) => {
            if let Err(e) = run_program_file(path.into()) {
                eprintln!("{}", e);
            }
        }
    }
    Ok(())
}

fn run_program_file(path: String) -> Result<(), MonkeyError> {
    let reader = BufReader::new(File::open(path)?);
    let lexer = lexer::Lexer::new(Box::new(reader))?;
    let parsed_program = parser::Parser::new(lexer)?.parse_program()?;
    interpreter::run_program(parsed_program)?;
    Ok(())
}
