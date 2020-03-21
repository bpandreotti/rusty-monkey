pub mod token;
pub mod lexer;
pub mod repl;
pub mod ast;
pub mod parser;

use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    repl::start()?;
    Ok(())
}
