mod token;
mod lexer;
mod repl;

use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {

    println!("Hello! This is the Monkey programming language!");
    repl::start()?;

    Ok(())
}
