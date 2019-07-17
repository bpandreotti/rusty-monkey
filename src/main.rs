mod token;
mod lexer;
mod repl;
mod ast;
mod parser;

use std::error::Error;

use parser::*;
use lexer::*;

fn main() -> Result<(), Box<dyn Error>> {
    
    let lex = Lexer::new("if (x < y) { x + 5 } else { y + 5 }".into());
    let mut pars = Parser::new(lex);

    println!("{:#?}", pars.parse_program());

    Ok(())
}
