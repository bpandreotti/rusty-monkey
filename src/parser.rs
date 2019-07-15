use crate::token::*;
use crate::lexer::*;
use crate::ast::*;

use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct ParserError(String);

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for ParserError {}

impl From<&str> for ParserError {
    fn from(s: &str) -> ParserError {
        ParserError(s.into())
    }
}

pub type ParserResult<T> = Result<T, ParserError>;

type PrefixParseFn = fn(&mut Parser) -> ParserResult<Expression>;
type InfixParseFn = fn(&mut Parser, Expression) -> ParserResult<Expression>;

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

    pub fn parse_program(&mut self) -> ParserResult<Vec<Statement>> {
        let mut program: Vec<Statement> = Vec::new();

        while self.current_token != Token::EOF {
            let statement = self.parse_statement()?;
            program.push(statement);
            self.read_token();
        }

        Ok(program)
    }

    fn parse_statement(&mut self) -> ParserResult<Statement> {
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

    fn parse_let_statement(&mut self) -> ParserResult<LetStatement> {
        if let Token::Identifier(iden) = &self.peek_token {
            let name = iden.clone(); // We have to clone this here to satisfy the borrow checker.

            self.read_token();
            if self.peek_token != Token::Assign {
                return Err("Expected `=` token".into());
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
            Err("Expected identifier token".into())
        }
    }

    fn parse_return_statement(&mut self) -> ParserResult<Expression> {
        self.read_token();
        // @TODO: Same thing as in `parse_let_statement`.
        while self.current_token != Token::Semicolon && self.current_token != Token::EOF {
            self.read_token();
        }

        Ok(Expression::Nil)
    }

    fn parse_expression_statement(&mut self) -> ParserResult<Expression> {
        let exp = self.parse_expression(Precedence::Lowest)?;

        if self.peek_token == Token::Semicolon {
            self.read_token();
        }

        Ok(exp)
    }

    fn parse_expression(&mut self, precedence: Precedence) -> ParserResult<Expression> {
        let prefix_parse_function = Parser::get_prefix_parse_function(&self.current_token);

        match prefix_parse_function {
            Some(function) => (function)(self),
            None => Err("No prefix parse function found for current token".into()),
        }
    }

    fn parse_prefix_expression(&mut self) -> ParserResult<Expression> {
        // @TODO: This is pretty ugly. I could parse the right side expression before I match, but
        // for that I would have to remember the prefix token, which means I would have to
        // #[derive(Clone)] for Token.
        match &self.current_token {
            Token::Bang => {
                self.read_token();
                let right_side = Box::new(self.parse_expression(Precedence::Prefix)?);
                Ok(Expression::PrefixBang(right_side))
            },
            Token::Minus => {
                self.read_token();
                let right_side = Box::new(self.parse_expression(Precedence::Prefix)?);
                Ok(Expression::PrefixMinus(right_side))
            },
            _ => Err("Trying to parse prefix expression, but current token is not `Token::Bang` \
                     or `Token::Minus`. This error should never happen.".into()),
        }
    }

    fn parse_identifier(&mut self) -> ParserResult<Expression> {
        match &self.current_token {
            Token::Identifier(s) => Ok(Expression::Identifier(s.clone())),
            _ => Err("Trying to parse identifier, but current token is not `Token::Identifier`. \
                     This error should never happen.".into()),
        }
    }

    fn parse_int_literal(&mut self) -> ParserResult<Expression> {
        match &self.current_token {
            Token::Int(x) => Ok(Expression::IntLiteral(*x)),
            _ => Err("Trying to parse int literal, but current token is not `Token::Int`. This \
                     error should never happen.".into()),
        }
    }

    fn get_prefix_parse_function(token: &Token) -> Option<PrefixParseFn> {
        match token {
            Token::Identifier(_) => Some(Parser::parse_identifier),
            Token::Int(_) => Some(Parser::parse_int_literal),
            Token::Bang | Token::Minus => Some(Parser::parse_prefix_expression),
            _ => None,
        }
    }

    fn get_infix_parse_function(token: Token) -> Option<InfixParseFn> {
        match token {
            _ => None,
        }
    }
}

