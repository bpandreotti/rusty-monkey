// @TODO: Add tests for this module.
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
type InfixParseFn = fn(&mut Parser, Box<Expression>) -> ParserResult<Expression>;

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

    fn expect_token(&mut self, expected: Token) -> ParserResult<()> {
        if std::mem::discriminant(&self.peek_token) != std::mem::discriminant(&expected) {
            Err(ParserError(format!(
                "Expected {} token, got {}.",
                expected.type_str(),
                self.peek_token.type_str()
            )))
        } else {
            self.read_token();
            Ok(())
        }
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
            self.expect_token(Token::Assign)?;

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
            Err(
                ParserError(format!("Expected literal token, got {}.", self.peek_token.type_str()))
            )
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

    fn parse_block_statement(&mut self) -> ParserResult<Vec<Statement>> {
        if self.current_token != Token::OpenBrace {
            return Err("Trying to parse block statement, but current token is not \
                       `Token::OpenBrace`. This error should never happen.".into())
        }
        self.read_token();

        let mut statements = Vec::new();
        while self.current_token != Token::CloseBrace {
            statements.push(self.parse_statement()?);
            self.read_token();
        }
        
        Ok(statements)
    }

    fn parse_expression(&mut self, precedence: Precedence) -> ParserResult<Expression> {
        let prefix_parse_fn = Parser::get_prefix_parse_function(&self.current_token);

        let mut left_expression = match prefix_parse_fn {
            Some(function) => function(self),
            None => {
                Err(ParserError(format!(
                    "No prefix parse function found for current token ({}).",
                    self.current_token.type_str()
                )))
            },
        }?;

        while self.peek_token != Token::Semicolon
            && precedence < Parser::get_precedence(&self.peek_token)
        {
            let infix_parse_fn = Parser::get_infix_parse_function(&self.peek_token);

            match infix_parse_fn {
                Some(function) => {
                    self.read_token();
                    left_expression = function(self, Box::new(left_expression))?;
                }
                None => break,
            }
        }

        Ok(left_expression)
    }

    fn parse_prefix_expression(&mut self) -> ParserResult<Expression> {
        match &self.current_token {
            Token::Bang | Token::Minus => {
                let operator = self.current_token.clone();
                self.read_token();
                let right_side = Box::new(self.parse_expression(Precedence::Prefix)?);
                Ok(Expression::PrefixExpression(operator, right_side))
            },
            _ => Err("Trying to parse prefix expression, but current token is not `Token::Bang` \
                     or `Token::Minus`. This error should never happen.".into()),
        }
    }

    fn parse_infix_expression(&mut self, left_side: Box<Expression>) -> ParserResult<Expression> {
        let operator = self.current_token.clone();
        let precedence = Parser::get_precedence(&operator);
        self.read_token();
        let right_side = Box::new(self.parse_expression(precedence)?);

        Ok(Expression::InfixExpression(left_side, operator, right_side))
    }

    fn parse_if_expression(&mut self) -> ParserResult<Expression> {
        self.read_token(); // Consume `if` token

        // Maybe it would be nice to make the parenthesis around the if condition optional, like
        // Rust. It doesn't appear to cause any problems for the parser.
        // self.expect_token(Token::OpenParen)?;
        let condition = self.parse_expression(Precedence::Lowest)?;
        // self.expect_token(Token::CloseParen)?;        
        
        self.expect_token(Token::OpenBrace)?;        
        let consequence = self.parse_block_statement()?;

        let alternative = if self.peek_token == Token::Else {
            self.read_token();
            self.expect_token(Token::OpenBrace)?;
            self.parse_block_statement()?
        } else {
            Vec::new()
        };

        Ok(Expression::IfExpression {
            condition: Box::new(condition),
            consequence,
            alternative,
        })
    }

    fn parse_grouped_expression(&mut self) -> ParserResult<Expression> {
        self.read_token();
        let exp = self.parse_expression(Precedence::Lowest)?;

        self.expect_token(Token::CloseParen)?;
        Ok(exp)
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

    fn parse_boolean(&mut self) -> ParserResult<Expression> {
        match &self.current_token {
            Token::True => Ok(Expression::Boolean(true)),
            Token::False => Ok(Expression::Boolean(false)),
            _ => Err("Trying to parse boolean, but current token is not `Token::True` or \
                     `Token::False`. This error should never happen.".into()),
        }
    }

    fn get_prefix_parse_function(token: &Token) -> Option<PrefixParseFn> {
        match token {
            Token::Identifier(_) => Some(Parser::parse_identifier),
            Token::Int(_) => Some(Parser::parse_int_literal),
            Token::True | Token::False => Some(Parser::parse_boolean),
            Token::Bang | Token::Minus => Some(Parser::parse_prefix_expression),
            Token::OpenParen => Some(Parser::parse_grouped_expression),
            Token::If => Some(Parser::parse_if_expression),
            _ => None,
        }
    }

    fn get_infix_parse_function(token: &Token) -> Option<InfixParseFn> {
        match token {
            Token::Equals
            | Token::NotEquals
            | Token::LessThan
            | Token::LessEq
            | Token::GreaterThan
            | Token::GreaterEq
            | Token::Plus
            | Token::Minus
            | Token::Slash
            | Token::Asterisk => Some(Parser::parse_infix_expression),
            _ => None,
        }
    }

    fn get_precedence(token: &Token) -> Precedence {
        use Token::*;
        match token {
            Equals | NotEquals                          => Precedence::Equals,
            LessThan | LessEq | GreaterThan | GreaterEq => Precedence::LessGreater,
            Plus | Minus                                => Precedence::Sum,
            Slash | Asterisk                            => Precedence::Product,
            _                                           => Precedence::Lowest,
        }
    }
}
