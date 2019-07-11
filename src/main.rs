mod token;
mod lexer;

use lexer::Lexer;
use token::Token;

fn main() {
    let input = r#"
        let five = 5;
        let ten = 10;

        let add = fn(x, y) {
            x + y;
        };

        let result = add(five, ten);
    "#.into();


    let mut lexer = Lexer::new(input);
    let mut tk = lexer.next_token();
    while tk != Token::EOF {
        println!("{:?}", tk);
        tk = lexer.next_token();
    }
}
