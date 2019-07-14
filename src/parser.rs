use crate::token::*;
use crate::lexer::*;
use crate::ast::*;

type PrefixParseFn = fn(&mut Parser) -> Result<Expression, ()>;
type InfixParseFn = fn(&mut Parser, Expression) -> Result<Expression, ()>;

#[derive(PartialEq, PartialOrd)]
enum Precedence {
    Lowest,
    Equals,
    LessGreater,
    Sum,
    Product,
    Prefix,
    Call,
}

pub struct Parser {
    lexer: Lexer,
    current_token: Token,
    peek_token: Token,
}

impl Parser {
    pub fn new(mut lexer: Lexer) -> Parser {
        let first_token = lexer.next_token();
        let second_token = lexer.next_token();
        Parser {
            lexer: lexer,
            current_token: first_token,
            peek_token: second_token,
        }
    }

    pub fn read_token(&mut self) {
        // Little trick to move the borrowed value without having to clone anything. I'm
        // effectively doing this:
        //   self.current_token = self.peek_token;
        //   self.peek_token = self.lexer.next_token();
        // but this way the borrow checker is pleased.
        self.current_token = std::mem::replace(&mut self.peek_token, self.lexer.next_token());
    }

    pub fn parse_program(&mut self) -> Result<Vec<Statement>, ()> {
        let mut program: Vec<Statement> = Vec::new();

        while self.current_token != Token::EOF {
            let statement = self.parse_statement()?;
            program.push(statement);
            self.read_token();
        }

        Ok(program)
    }

    fn parse_statement(&mut self) -> Result<Statement, ()> {
        match self.current_token {
            Token::Let => {
                let st = Box::new(self.parse_let_statement()?);
                Ok(Statement::Let(st))
            },
            Token::Return => {
                let st = Box::new(self.parse_return_statement()?);
                Ok(Statement::Return(st))
            },
            _ => {
                let st = Box::new(self.parse_expression_statement()?);
                Ok(Statement::ExpressionStatement(st))
            }
        }
    }

    fn parse_let_statement(&mut self) -> Result<LetStatement, ()> {
        if let Token::Identifier(iden) = &self.peek_token {
            let name = iden.clone(); // We have to clone this here to satisfy the borrow checker.

            self.read_token();
            if self.peek_token != Token::Assign {
                return Err(());
            }

            // @TODO: Since we can't yet parse expressions, we're ignoring the actual value of the
            // let statement. I'm just skipping tokens until we find a semicolon.
            while self.current_token != Token::Semicolon && self.current_token != Token::EOF {
                self.read_token();
            }

            Ok(LetStatement {
                identifier: name,
                value: Expression::Nil,
            })
        } else {
            Err(())
        }
    }

    fn parse_return_statement(&mut self) -> Result<Expression, ()> {
        self.read_token();
        // @TODO: Same thing as in `parse_let_statement`.
        while self.current_token != Token::Semicolon && self.current_token != Token::EOF {
            self.read_token();
        }

        Ok(Expression::Nil)
    }

    fn parse_expression_statement(&mut self) -> Result<Expression, ()> {
        let exp = self.parse_expression(Precedence::Lowest)?;

        if self.peek_token == Token::Semicolon {
            self.read_token();
        }

        Ok(exp)
    }

    fn parse_expression(&mut self, precedence: Precedence) -> Result<Expression, ()> {
        let prefix_parse_function = Parser::get_prefix_parse_function(&self.current_token);

        match prefix_parse_function {
            Some(function) => (function)(self),
            None => Err(()),
        }
    }

    fn get_prefix_parse_function(token: &Token) -> Option<PrefixParseFn> {
        match token {
            // @TODO: Maybe making these closures actual methods would make the code cleaner.
            Token::Identifier(_) => Some(|parser| {
                match &parser.current_token {
                    Token::Identifier(s) => Ok(Expression::Identifier(s.clone())),
                    _ => panic!(),
                }
            }),

            Token::Int(_) => Some(|parser| {
                match &parser.current_token {
                    Token::Int(x) => Ok(Expression::IntLiteral(*x)),
                    _ => panic!(),
                }
            }),

            _ => None,
        }
    }

    fn get_infix_parse_function(token: Token) -> Option<InfixParseFn> {
        match token {
            _ => None,
        }
    }
}
