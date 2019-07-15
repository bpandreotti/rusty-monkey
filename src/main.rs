mod token;
mod lexer;
mod repl;
mod ast;
mod parser;

use std::error::Error;

use parser::*;
use lexer::*;

fn main() -> Result<(), Box<dyn Error>> {
    
    let lex = Lexer::new("3 + 4 * 5 == 3 * 1 + 4 * 5".into());
    let mut pars = Parser::new(lex);

    println!("{:#?}", pars.parse_program());

    Ok(())
}
