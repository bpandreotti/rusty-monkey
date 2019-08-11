mod token;
mod lexer;
mod repl;
mod ast;
mod parser;

use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    repl::start()?;
    Ok(())
}
