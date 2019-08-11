mod token;
mod lexer;
mod repl;
mod ast;
mod parser;

use std::error::Error;

use parser::*;
use lexer::*;

fn main() -> Result<(), Box<dyn Error>> {
    let lex = Lexer::new("add(a, b, 1, 2 * 3, 4 + 5, add(6, 7 * 8))".into());
    let mut pars = Parser::new(lex);

    println!("{:#?}", pars.parse_program());

    Ok(())
}
