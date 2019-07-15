mod token;
mod lexer;
mod repl;
mod ast;
mod parser;

use std::error::Error;

use parser::*;
use lexer::*;

fn main() -> Result<(), Box<dyn Error>> {

    let lex = Lexer::new("!foo; -5; !--!!43".into());
    let mut pars = Parser::new(lex);

    println!("{:#?}", pars.parse_program());

    Ok(())
}
