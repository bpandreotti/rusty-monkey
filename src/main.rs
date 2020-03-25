pub mod ast;
pub mod environment;
pub mod eval;
pub mod lexer;
pub mod object;
pub mod parser;
pub mod repl;
pub mod token;

use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    repl::start()?;
    Ok(())
}
