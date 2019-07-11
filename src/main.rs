mod token;
mod lexer;

use lexer::Lexer;

fn main() {
    let input = "Hello, world!".into();
    let mut lexer = Lexer::new(input);
    
    while let Some(c) = lexer.current_char {
        println!("{}", c);
        lexer.read_char();
    }
}
