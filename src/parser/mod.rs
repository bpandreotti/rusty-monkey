pub mod ast;
#[cfg(test)] mod tests;

use crate::error::*;
use crate::lexer::{Lexer, token::Token};
use ast::*;

use std::mem;

pub fn parse(input: String) -> MonkeyResult<Vec<NodeStatement>> {
    let lex = Lexer::from_string(input)?;
    let mut pars = Parser::new(lex)?;
    pars.parse_program()
}

type PrefixParseFn = fn(&mut Parser) -> MonkeyResult<NodeExpression>;
type InfixParseFn = fn(&mut Parser, Box<NodeExpression>) -> MonkeyResult<NodeExpression>;

#[derive(PartialEq, PartialOrd)]
enum Precedence {
    Lowest,
    Equals,
    LessGreater,
    Sum,
    Product,
    Exponent,
    Prefix,
    Call,
    Index,
}

pub struct Parser {
    lexer: Lexer,
    current_token: Token,
    peek_token: Token,
    position: (usize, usize),
}

impl Parser {
    pub fn new(mut lexer: Lexer) -> MonkeyResult<Parser> {
        let current_token = lexer.next_token()?;
        let position = lexer.token_position;
        let peek_token = lexer.next_token()?;
        Ok(Parser { lexer, current_token, peek_token, position })
    }

    /// Parses the program passed to the lexer. Reads tokens from the lexer until reaching EOF, and
    /// outputs the parsed statements into a `Vec`.
    pub fn parse_program(&mut self) -> MonkeyResult<Vec<NodeStatement>> {
        let mut program: Vec<NodeStatement> = Vec::new();

        while self.current_token != Token::EOF {
            let statement = self.parse_statement()?;
            program.push(statement);
            self.read_token()?;
        }

        Ok(program)
    }

    /// Reads a token from the lexer and updates `self.current_token` and `self.peek_token`.
    fn read_token(&mut self) -> MonkeyResult<()> {
        // Because we eagerly call `self.lexer.next_token` to get the peek token, the lexer
        // position is always one token ahead of the parser -- that is, `self.lexer.token_position`
        // is the position of `self.peek_token`. Therefore, we have to update the parser
        // position before calling `self.lexer.next_token`
        self.position = self.lexer.token_position;
        // Little trick to move the borrowed value without having to clone anything. I'm
        // effectively doing this:
        //   self.current_token = self.peek_token;
        //   self.peek_token = self.lexer.next_token();
        // but this way the borrow checker is pleased.
        self.current_token = mem::replace(&mut self.peek_token, self.lexer.next_token()?);
        Ok(())
    }

    /// Checks if `self.peek_token` has the same discriminant as the token passed. If so, reads
    /// this token from the lexer. Otherwise, if `self.peek_token` is not the expected token,
    /// does nothing and returns an error.
    fn expect_token(&mut self, expected: Token) -> MonkeyResult<()> {
        if mem::discriminant(&self.peek_token) == mem::discriminant(&expected) {
            self.read_token()?;
            Ok(())
        } else {
            Err(parser_err(
                self.position,
                ParserError::UnexpectedToken(expected, self.peek_token.clone())
            ))
        }
    }

    /// Same as `expect_token`, but also accepts `Token::EOF`. If it does find EOF, doesn't
    /// consume it.
    fn expect_token_or_eof(&mut self, expected: Token) -> MonkeyResult<()> {
        if mem::discriminant(&self.peek_token) == mem::discriminant(&expected) {
            self.read_token()?;
            Ok(())            
        } else if self.peek_token == Token::EOF {
            Ok(())
        } else {
            Err(parser_err(
                self.position,
                ParserError::UnexpectedToken(expected, self.peek_token.clone())
            ))
        }
    }

    /// Same as `expect_token`, but allows for more than one possibility.
    fn expect_token_multiple(&mut self, possibilities: &'static [Token]) -> MonkeyResult<()> {
        let found = possibilities
            .iter()
            .any(|tk| mem::discriminant(&self.peek_token) == mem::discriminant(tk));
        if !found {
            Err(parser_err(
                self.position,
                ParserError::UnexpectedTokenMultiple {
                    possibilities,
                    got: self.peek_token.clone(),
                }
            ))
        } else {
            self.read_token()?;
            Ok(())
        }
    }

    /// Same as `expect_token`, but if it doesn't find the expected token, just does nothing.
    fn consume_optional_token(&mut self, expected: Token) -> MonkeyResult<()> {
        if mem::discriminant(&self.peek_token) == mem::discriminant(&expected) {
            self.read_token()?;
        }
        Ok(())
    }

    /// Parses a statement from the program. A statement can be a "let" statement, a "return"
    /// statement, an expression statement, or a block of statements. May return an error if
    /// parsing fails.
    fn parse_statement(&mut self) -> MonkeyResult<NodeStatement> {
        let position = self.position;
        let statement = match self.current_token {
            Token::Let => {
                let let_st = Box::new(self.parse_let_statement()?);
                Statement::Let(let_st)
            }
            Token::Return => {
                let exp = Box::new(self.parse_return_statement()?);
                Statement::Return(exp)
            }
            _ => {
                let exp = Box::new(self.parse_expression_statement()?);
                Statement::ExpressionStatement(exp)
            }
        };
        Ok(NodeStatement { position, statement })
    }

    /// Parses a "let" statement. Expects an "=" token, followed by an identifier and finally an
    /// expression. Returns an error if any of those steps fail. Doesn't check if
    /// `self.current_token` is a "let" token. Must end in a semicolon.
    fn parse_let_statement(&mut self) -> MonkeyResult<LetStatement> {
        self.read_token()?; // Read identifier token
        if let Token::Identifier(iden) = &self.current_token {
            // We have to clone this here to satisfy the borrow checker
            let identifier = iden.clone();

            self.expect_token(Token::Assign)?; // Expect "=" token
            self.read_token()?; // Read first token from the expression

            // At this point, self.current_token is the first token in the expression
            let value = self.parse_expression(Precedence::Lowest)?;
            self.expect_token_or_eof(Token::Semicolon)?;
            Ok((identifier, value))
        } else {
            Err(parser_err(
                self.position,
                ParserError::UnexpectedToken(
                    Token::Identifier("".into()),
                    self.current_token.clone()
                ),
            ))
        }
    }

    /// Parses a "return" statement. Expects a valid expression, and returns an error if its
    /// parsing fails. Doesn't check if `self.current_token` is a "return" token.  Must end in a
    /// semicolon.
    fn parse_return_statement(&mut self) -> MonkeyResult<NodeExpression> {
        let return_value = if self.peek_token == Token::Semicolon {
            // In case of no return value, we return nil
            NodeExpression {
                position: self.position,
                expression: Expression::Nil
            }
        } else {
            self.read_token()?; // Read first token from the expression
            self.parse_expression(Precedence::Lowest)?
        };
        self.expect_token_or_eof(Token::Semicolon)?;
        Ok(return_value)
    }

    /// Parses an expression statement, returns an error if parsing fails.  Must end in a
    /// semicolon, unless either:
    /// * The expression is an "if" expression, a function literal or a block expression.
    /// * The first token after the expression is a "}" token, meaning the expression is the last
    /// expression in the current block.
    fn parse_expression_statement(&mut self) -> MonkeyResult<NodeExpression> {
        let exp = self.parse_expression(Precedence::Lowest)?;
        match exp.expression {
            Expression::IfExpression { .. }
            | Expression::FunctionLiteral { .. }
            | Expression::BlockExpression { .. } => {
                // In these three cases, the semicolon is optional
                self.consume_optional_token(Token::Semicolon)?
            }
            _ => {
                // If we are at the end of a block (peek token is "}") or at the end of the program
                // (peek token is EOF), the semicolon is also optional
                if self.peek_token != Token::CloseCurlyBrace && self.peek_token != Token::EOF {
                    self.expect_token(Token::Semicolon)?
                }
            }
        }
        Ok(exp)
    }

    /// Parses a block of statements. A block of statements must be enclosed by curly braces.
    /// Returns an error if the parsing of any statement inside fails. Doesn't check if
    /// `self.current_token` is "{".
    fn parse_block_statement(&mut self) -> MonkeyResult<Vec<NodeStatement>> {
        self.read_token()?;
        let mut statements = Vec::new();
        while self.current_token != Token::CloseCurlyBrace {
            statements.push(self.parse_statement()?);
            self.read_token()?;
        }

        Ok(statements)
    }

    /// Parses an expression. First, parses an expression using a prefix parse function. This step
    /// parses all expressions except for infix operator expressions and function call expressions.
    /// After that, attempts to use the parsed expression as the left side of another expression,
    /// now using an infix parse function. This step deals with the aforementioned remaining cases.
    /// Returns an error if no prefix parse function is encountered, or the parsing fails.
    fn parse_expression(&mut self, precedence: Precedence) -> MonkeyResult<NodeExpression> {
        let prefix_parse_fn = match Parser::get_prefix_parse_function(&self.current_token) {
            Some(f) => f,
            None => return Err(parser_err(
                self.position,
                ParserError::NoPrefixParseFn(self.current_token.clone()),
            )),
        };

        let mut left_expression = prefix_parse_fn(self)?;

        while self.peek_token != Token::Semicolon
            && precedence < Parser::get_precedence(&self.peek_token)
        {
            let infix_parse_fn = Parser::get_infix_parse_function(&self.peek_token);

            match infix_parse_fn {
                Some(f) => {
                    self.read_token()?;
                    left_expression = f(self, Box::new(left_expression))?;
                }
                // This is only reached if the peek token has a higher precedence than the current
                // token, and if it doesn't have an infix parse function associated with it.
                // Currently, all tokens that don't have infix parse functions have the lowest
                // precedence, so that is impossible
                None => unreachable!(),
            }
        }
        
        Ok(left_expression)
    }

    /// Parses a prefix expression. These are composed of a prefix operator (like "-" or "!") and
    /// an expression for the right side. May return an error if the parsing of the right side
    /// fails. Panics if `self.current_token` is not a valid prefix operator.
    fn parse_prefix_expression(&mut self) -> MonkeyResult<NodeExpression> {
        let position = self.position;
        match &self.current_token {
            Token::Bang | Token::Minus => {
                let operator = self.current_token.clone();
                self.read_token()?;
                let right_side = Box::new(self.parse_expression(Precedence::Prefix)?);
                Ok(NodeExpression {
                    position,
                    expression: Expression::PrefixExpression(operator, right_side),
                })
            }
            _ => panic!(),
        }
    }

    /// Parses an infix expression. These are composed of a left side expression, an infix operator
    /// (like "+" or ">") and a right side expression. Takes an already parsed left side and parses
    /// the right side using the operator's precedence. `self.current_token` must be a valid
    /// operator token. Returns an error if the right side parsing fails.
    fn parse_infix_expression(&mut self, left_side: Box<NodeExpression>) -> MonkeyResult<NodeExpression> {
        let position = self.position;
        let operator = self.current_token.clone();
        let precedence = Parser::get_precedence(&operator);
        self.read_token()?;
        let right_side = Box::new(self.parse_expression(precedence)?);
        Ok(NodeExpression {
            position,
            expression: Expression::InfixExpression(left_side, operator, right_side)
        })
    }

    /// Parses an "if" expression. These are composed of the "if" keyword, a condition expression,
    /// and a block of statements as a consequence. Optionally, there can be an "else" branch,
    /// composed of the "else" keyword and another block of statements. May return an error if
    /// parsing fails at any point. Doesn't check if `self.current_token` is an "if" token.
    fn parse_if_expression(&mut self) -> MonkeyResult<NodeExpression> {
        let position = self.position;
        self.read_token()?; // Read first token from the condition expression
        let condition = self.parse_expression(Precedence::Lowest)?;

        self.expect_token(Token::OpenCurlyBrace)?;
        let consequence = self.parse_block_statement()?;

        let alternative = if self.peek_token == Token::Else {
            self.read_token()?; // Consume "else" token
            self.expect_token_multiple(&[Token::OpenCurlyBrace, Token::If])?;
            match self.current_token {
                Token::OpenCurlyBrace => self.parse_block_statement()?,
                // This call to `self.parse_statement` is guaranteed to result in an expression
                // statement with an if expression inside, because the current token is `if`.
                Token::If => vec![self.parse_statement()?],
                _ => unreachable!(),
            }
        } else {
            Vec::new()
        };
        
        Ok(NodeExpression {
            position,
            expression: Expression::IfExpression {
                condition: Box::new(condition),
                consequence,
                alternative,
            }
        })
    }

    /// Parses a function literal. Expects a valid function parameter list enclosed by parentheses,
    /// followed by a block of statements. May return an error if parsing fails. Doesn't check if
    /// `self.current_token` is an "fn" token.
    fn parse_function_literal(&mut self) -> MonkeyResult<NodeExpression> {
        let position = self.position;
        self.expect_token(Token::OpenParen)?;
        let parameters = self.parse_function_parameters()?;
        self.expect_token(Token::OpenCurlyBrace)?;
        let body = self.parse_block_statement()?;

        Ok(NodeExpression {
            position,
            expression: Expression::FunctionLiteral { parameters, body }
        })
    }

    /// Parses a function call expression. The function must be already parsed, and passed as an
    /// expression. Expects a valid list of call arguments. Doesn't check if `self.current_token`
    /// is an "(" token. May return an error if parsing fails.
    fn parse_call_expression(&mut self, function: Box<NodeExpression>) -> MonkeyResult<NodeExpression> {
        let position = self.position;
        let arguments = self.parse_expression_list(Token::CloseParen)?;
        Ok(NodeExpression {
            position,
            expression: Expression::CallExpression { function, arguments }
        })
    }

    /// Parses a function parameter list. These are a list of identifiers, enclosed by parentheses
    /// and separated by commas. There should be no trailing comma. Returns an error if the parser
    /// encounters an unexpected token while parsing. Doesn't check if `self.current_token` is an
    /// "(" token.
    fn parse_function_parameters(&mut self) -> MonkeyResult<Vec<String>> {
        let mut params = Vec::new();

        // In case of empty parameter list
        if self.peek_token == Token::CloseParen {
            self.read_token()?;
            return Ok(params);
        }

        self.expect_token(Token::Identifier("".into()))?;

        // The `expect_token` calls assure that the while condition is always true. The while loop
        // only exits on the `break`, if the parser encounters a ")" token, or if there are any
        // errors
        while let Token::Identifier(iden) = &self.current_token {
            params.push(iden.clone());
            self.expect_token_multiple(&[Token::Comma, Token::CloseParen])?;
            match &self.current_token {
                Token::CloseParen => return Ok(params),
                Token::Comma => {
                    self.expect_token(Token::Identifier("".into()))?;
                }
                _ => unreachable!(),
            }
        }

        self.expect_token(Token::CloseParen)?;
        Ok(params)
    }

    /// Parses a grouped expression, that is, an expression enclosed by parentheses. This only has
    /// the effect of parsing the inner expression with a lower precedence. Returns an error if the
    /// parsing of the inner expression fails, or if the parser doesn't encounter the ")" token.
    fn parse_grouped_expression(&mut self) -> MonkeyResult<NodeExpression> {
        self.read_token()?;
        let exp = self.parse_expression(Precedence::Lowest)?;
        self.expect_token(Token::CloseParen)?;
        Ok(exp)
    }

    /// Parses an identifier token into an identifier expression.
    fn parse_identifier(&mut self) -> MonkeyResult<NodeExpression> {
        match &self.current_token {
            Token::Identifier(s) => Ok(NodeExpression {
                position: self.position,
                expression: Expression::Identifier(s.clone()),
            }),
            _ => panic!(),
        }
    }

    /// Parses an integer token into an integer literal expression.
    fn parse_int_literal(&mut self) -> MonkeyResult<NodeExpression> {
        match &self.current_token {
            Token::Int(x) => Ok(NodeExpression {
                position: self.position,
                expression: Expression::IntLiteral(*x),
            }),
            _ => panic!(),
        }
    }

    /// Parses a string token into a string literal expression.
    fn parse_string_literal(&mut self) -> MonkeyResult<NodeExpression> {
        match &self.current_token {
            Token::Str(s) => Ok(NodeExpression {
                position: self.position,
                expression: Expression::StringLiteral(s.clone()),
            }),
            _ => panic!(),
        }
    }

    /// Parses a boolean token into a boolean literal expression.
    fn parse_boolean(&mut self) -> MonkeyResult<NodeExpression> {
        let value = match &self.current_token {
            Token::True => true,
            Token::False => false,
            _ => panic!()
        };
        Ok(NodeExpression {
            position: self.position,
            expression: Expression::Boolean(value),
        })
    }

    /// Parses the "nil" keyword into the null value.
    fn parse_nil(&mut self) -> MonkeyResult<NodeExpression> {
        if self.current_token == Token::Nil {
            Ok(NodeExpression {
                position: self.position,
                expression: Expression::Nil
            })
        } else {
            panic!()
        }
    }

    /// Parses an array literal. Doesn't check if `self.current_token` is an "[" token.
    fn parse_array_literal(&mut self) -> MonkeyResult<NodeExpression> {
        let elements = self.parse_expression_list(Token::CloseSquareBracket)?;
        Ok(NodeExpression {
            position: self.position,
            expression: Expression::ArrayLiteral(elements)
        })
    }

    /// Parses a array indexing expression. The array must be already parsed, and passed as an
    /// expression. Expects an expression as the index. Doesn't check if `self.current_token`
    /// is an "[" token. May return an error if parsing fails.
    fn parse_index_expression(&mut self, left: Box<NodeExpression>) -> MonkeyResult<NodeExpression> {
        self.read_token()?; // Read first token of index expression
        let index = self.parse_expression(Precedence::Lowest)?;
        self.expect_token(Token::CloseSquareBracket)?;
        Ok(NodeExpression {
            position: self.position,
            expression: Expression::IndexExpression(left, Box::new(index))
        })
    }

    /// Parses a hash literal. Doesn't check if `self.current_token` is an "#{" token.
    fn parse_hash_literal(&mut self) -> MonkeyResult<NodeExpression> {
        let position = self.position;
        let mut entries = Vec::new();
        if self.peek_token == Token::CloseCurlyBrace {
            self.read_token()?;
            return Ok(NodeExpression {
                position,
                expression: Expression::HashLiteral(entries)
            });
        }

        self.read_token()?;
        entries.push(self.parse_hash_entry()?);
        while self.peek_token == Token::Comma {
            self.read_token()?;
            self.read_token()?;
            entries.push(self.parse_hash_entry()?);
        }
        self.expect_token(Token::CloseCurlyBrace)?;
        Ok(NodeExpression {
            position,
            expression: Expression::HashLiteral(entries)
        })
    }

    /// Parses a hash entry, that is, two expressions separated by a ":" token.
    fn parse_hash_entry(&mut self) -> MonkeyResult<(NodeExpression, NodeExpression)> {
        let key = self.parse_expression(Precedence::Lowest)?;
        self.expect_token(Token::Colon)?;
        self.read_token()?;
        let value = self.parse_expression(Precedence::Lowest)?;
        Ok((key, value))
    }

    fn parse_block_expression(&mut self) -> MonkeyResult<NodeExpression> {
        let position = self.position;
        let block = self.parse_block_statement()?;
        Ok(NodeExpression {
            position,
            expression: Expression::BlockExpression(block)
        })
    }

    /// Parses a list of expressions, separated by commas and ending on `closing_token`. There
    /// should be no trailing comma. May return an error if parsing of a list element fails, or if
    /// the parser encounters an unexpected token.
    fn parse_expression_list(&mut self, closing_token: Token) -> MonkeyResult<Vec<NodeExpression>> {
        let mut list = Vec::new();
        // In case of empty expression list
        if self.peek_token == closing_token {
            self.read_token()?;
            return Ok(list);
        }

        self.read_token()?; // Read first token of expression
        list.push(self.parse_expression(Precedence::Lowest)?);
        while self.peek_token == Token::Comma {
            self.read_token()?; // Consume comma token
            self.read_token()?; // Read first token of expression
            list.push(self.parse_expression(Precedence::Lowest)?);
        }
        self.expect_token(closing_token)?;
        Ok(list)
    }

    /// Returns the prefix parse function associated with the given token.
    fn get_prefix_parse_function(token: &Token) -> Option<PrefixParseFn> {
        match token {
            Token::Identifier(_)        => Some(Parser::parse_identifier),
            Token::Int(_)               => Some(Parser::parse_int_literal),
            Token::Str(_)               => Some(Parser::parse_string_literal),
            Token::Bang | Token::Minus  => Some(Parser::parse_prefix_expression),
            Token::OpenParen            => Some(Parser::parse_grouped_expression),
            Token::OpenCurlyBrace       => Some(Parser::parse_block_expression),
            Token::OpenSquareBracket    => Some(Parser::parse_array_literal),
            Token::OpenHash             => Some(Parser::parse_hash_literal),
            Token::True | Token::False  => Some(Parser::parse_boolean),
            Token::If                   => Some(Parser::parse_if_expression),
            Token::Function             => Some(Parser::parse_function_literal),
            Token::Nil                  => Some(Parser::parse_nil),
            _ => None,
        }
    }

    /// Returns the infix parse function associated with the given token.
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
            | Token::Asterisk
            | Token::Exponent
            | Token::Modulo => Some(Parser::parse_infix_expression),
            Token::OpenParen => Some(Parser::parse_call_expression),
            Token::OpenSquareBracket => Some(Parser::parse_index_expression),
            _ => None,
        }
    }

    /// Returns the operator precedence associated with the given token.
    fn get_precedence(token: &Token) -> Precedence {
        use Token::*;
        match token {
            Equals | NotEquals                          => Precedence::Equals,
            LessThan | LessEq | GreaterThan | GreaterEq => Precedence::LessGreater,
            Plus | Minus                                => Precedence::Sum,
            Slash | Asterisk | Modulo                   => Precedence::Product,
            Exponent                                    => Precedence::Exponent,
            OpenParen                                   => Precedence::Call,
            OpenSquareBracket                           => Precedence::Index,
            _                                           => Precedence::Lowest,
        }
    }
}
