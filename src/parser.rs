use crate::token::*;
use crate::lexer::*;
use crate::ast::*;

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
            _ => Err(()),
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
}
