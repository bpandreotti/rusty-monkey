mod token;
mod lexer;
mod repl;
mod ast;
mod parser;

use std::error::Error;

use parser::*;
use lexer::*;

fn main() -> Result<(), Box<dyn Error>> {
    let lex = Lexer::new("fn (x, y, z) { x + y + z }".into());
    let mut pars = Parser::new(lex);

    println!("{:#?}", pars.parse_program());

    Ok(())
}
