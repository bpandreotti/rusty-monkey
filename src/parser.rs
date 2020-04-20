use crate::ast::*;
use crate::error::*;
use crate::lexer::*;
use crate::token::*;

use std::mem;

type PrefixParseFn = fn(&mut Parser) -> MonkeyResult<NodeExpression>;
type InfixParseFn = fn(&mut Parser, Box<NodeExpression>) -> MonkeyResult<NodeExpression>;

#[derive(PartialEq, PartialOrd)]
enum Precedence {
    Lowest,
    Equals,
    LessGreater,
    Sum,
    Product,
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
        if mem::discriminant(&self.peek_token) != mem::discriminant(&expected) {
            Err(parser_err(
                self.position,
                ParserError::UnexpectedToken(expected, self.peek_token.clone())
            ))
        } else {
            self.read_token()?;
            Ok(())
        }
    }

    /// Parses a statement from the program. A statement can be a "let" statement, a "return"
    /// statement, an expression statement, or a block of statements. May return an error if
    /// parsing fails.
    fn parse_statement(&mut self) -> MonkeyResult<NodeStatement> {
        let position = self.position;
        match self.current_token {
            Token::Let => {
                let let_st = Box::new(self.parse_let_statement()?);
                Ok(NodeStatement {
                    position,
                    statement: Statement::Let(let_st)
                })
            }
            Token::Return => {
                let exp = Box::new(self.parse_return_statement()?);
                Ok(NodeStatement {
                    position,
                    statement: Statement::Return(exp)
                })
            }
            // @TODO: Should block statements be expression statements with block expressions?
            Token::OpenCurlyBrace => {
                let block = self.parse_block_statement()?;
                Ok(NodeStatement {
                    position,
                    statement: Statement::BlockStatement(block)
                })
            }
            _ => {
                let exp = Box::new(self.parse_expression_statement()?);
                Ok(NodeStatement {
                    position,
                    statement: Statement::ExpressionStatement(exp)
                })
            }
        }
    }

    /// Parses a "let" statement. Expects an "=" token, followed by an identifier and finally an
    /// expression. Returns an error if any of those steps fail. Doesn't check if
    /// `self.current_token` is a "let" token.
    fn parse_let_statement(&mut self) -> MonkeyResult<LetStatement> {
        self.read_token()?; // Read identifier token
        if let Token::Identifier(iden) = &self.current_token {
            // We have to clone this here to satisfy the borrow checker
            let identifier = iden.clone();

            self.expect_token(Token::Assign)?; // Expect "=" token
            self.read_token()?; // Read first token from the expression

            // At this point, self.current_token is the first token in the expression
            let value = self.parse_expression(Precedence::Lowest)?;

            if self.peek_token == Token::Semicolon {
                self.read_token()?; // Consume optional semicolon
            }

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
    /// parsing fails. Doesn't check if `self.current_token` is a "return" token.
    fn parse_return_statement(&mut self) -> MonkeyResult<NodeExpression> {
        self.read_token()?; // Read first token from the expression
        let return_value = self.parse_expression(Precedence::Lowest)?;

        if self.peek_token == Token::Semicolon {
            self.read_token()?; // Consume optional semicolon
        }
        Ok(return_value)
    }

    /// Parses an expression statement, returns an error if parsing fails.
    fn parse_expression_statement(&mut self) -> MonkeyResult<NodeExpression> {
        let exp = self.parse_expression(Precedence::Lowest)?;
        if self.peek_token == Token::Semicolon {
            self.read_token()?; // Consume optional semicolon
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
            self.expect_token(Token::OpenCurlyBrace)?;
            self.parse_block_statement()?
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

            match &self.peek_token {
                Token::CloseParen => break,
                Token::Comma => {
                    self.read_token()?;
                    self.expect_token(Token::Identifier("".into()))?;
                }
                invalid => {
                    return Err(parser_err(
                        self.position,
                        ParserError::UnexpectedTokenMultiple {
                            possibilities: &[Token::Comma, Token::CloseParen],
                            got: invalid.clone(),
                        }
                    ));
                }
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
            | Token::Asterisk => Some(Parser::parse_infix_expression),
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
            Slash | Asterisk                            => Precedence::Product,
            OpenParen                                   => Precedence::Call,
            OpenSquareBracket                           => Precedence::Index,
            _                                           => Precedence::Lowest,
        }
    }
}

#[cfg(test)]
mod tests {
    // @TODO: Add tests for `parse_function_literal` and `parse_call_expression`
    // @TODO: Add tests for block expressions
    // @TODO: Add tests for parser position
    use super::*;
    use crate::lexer::Lexer;

    fn assert_parse(input: &str, expected: &[&str]) {
        let lex = Lexer::from_string(input.into()).unwrap();
        let mut pars = Parser::new(lex).unwrap();
        let output = pars.parse_program().expect("Parser error during test");
        assert_eq!(output.len(), expected.len());

        for i in 0..output.len() {
            assert_eq!(format!("{:?}", output[i]), expected[i]);
        }
    }

    fn assert_parse_fails(input: &str) {
        let lex = Lexer::from_string(input.into()).unwrap();
        let mut pars = Parser::new(lex).unwrap();
        let output = pars.parse_program();
        assert!(output.is_err());
    }

    #[test]
    fn test_literals() {
        let input = r#"
            0;
            17;
            true;
            false;
            nil;
            "brown is dark orange"
            "hello world";
            [];
            [0, false, nil];
            let hash = #{
                first : "entry",
                second : 1,
                nil : []
            }
        "#;
        let expected = [
            "ExpressionStatement(IntLiteral(0))",
            "ExpressionStatement(IntLiteral(17))",
            "ExpressionStatement(Boolean(true))",
            "ExpressionStatement(Boolean(false))",
            "ExpressionStatement(Nil)",
            "ExpressionStatement(StringLiteral(\"brown is dark orange\"))",
            "ExpressionStatement(StringLiteral(\"hello world\"))",
            "ExpressionStatement(ArrayLiteral([]))",
            "ExpressionStatement(ArrayLiteral([IntLiteral(0), Boolean(false), Nil]))",
            "Let((\"hash\", HashLiteral([(Identifier(\"first\"), StringLiteral(\"entry\")), \
            (Identifier(\"second\"), IntLiteral(1)), (Nil, ArrayLiteral([]))])))"
        ];
        assert_parse(input, &expected);
        assert_parse_fails("[a, b");
        assert_parse_fails("[nil,]");
    }

    #[test]
    fn test_index_expression() {
        let input = "
            a[0];
            [nil][0];
        ";
        let expected = [
            "ExpressionStatement(IndexExpression(Identifier(\"a\"), IntLiteral(0)))",
            "ExpressionStatement(IndexExpression(ArrayLiteral([Nil]), IntLiteral(0)))",
        ];
        assert_parse(input, &expected);

        assert_parse_fails("array[]");
        assert_parse_fails("array[i");
        assert_parse_fails("array[only, one, index, man]");
    }

    #[test]
    fn test_let_statements() {
        assert_parse(
            "let a = 1;",
            &["Let((\"a\", IntLiteral(1)))"],
        );
        assert_parse_fails("let 2 = 3;");
        assert_parse_fails("let foo whatever 3;");
        assert_parse_fails("let bar = ;");
        assert_parse_fails("let baz;");
    }

    #[test]
    fn test_return_statements() {
        // Not much to test here, to be honest
        assert_parse("return 0;", &["Return(IntLiteral(0))"]);
        assert_parse_fails("return;");
    }

    #[test]
    fn test_block_statements() {
        let input = "
            { let foo = 2; return 1; }
            { return 0 }
            {}
        ";
        let expected = [
            "BlockStatement([Let((\"foo\", IntLiteral(2))), Return(IntLiteral(1))])",
            "BlockStatement([Return(IntLiteral(0))])",
            "BlockStatement([])",
        ];
        assert_parse(input, &expected);

        assert_parse_fails("{ return 0");
    }

    #[test]
    fn test_prefix_expressions() {
        let input = "-5; !true; --!!-foo;";
        let expected = [
            "ExpressionStatement(PrefixExpression(Minus, IntLiteral(5)))",
            "ExpressionStatement(PrefixExpression(Bang, Boolean(true)))",
            "ExpressionStatement(PrefixExpression(Minus, PrefixExpression(Minus, PrefixExpression(\
            Bang, PrefixExpression(Bang, PrefixExpression(Minus, Identifier(\"foo\")))))))",
        ];
        assert_parse(input, &expected);

        assert_parse_fails("!;");
        assert_parse_fails("-");
    }

    #[test]
    fn test_infix_expressions() {
        let input = "1 + 2; 4 * 5 - 2 / 3; 1 >= 2 == 2 < 3 != true;";
        let expected = [
            "ExpressionStatement(InfixExpression(IntLiteral(1), Plus, IntLiteral(2)))",
            "ExpressionStatement(InfixExpression(InfixExpression(IntLiteral(4), Asterisk, \
            IntLiteral(5)), Minus, InfixExpression(IntLiteral(2), Slash, IntLiteral(3))))",
            "ExpressionStatement(InfixExpression(InfixExpression(InfixExpression(IntLiteral(1), \
            GreaterEq, IntLiteral(2)), Equals, InfixExpression(IntLiteral(2), LessThan, \
            IntLiteral(3))), NotEquals, Boolean(true)))",
        ];
        assert_parse(input, &expected);

        assert_parse_fails("1 + 2 -");
        assert_parse_fails("1 == + 2");
        assert_parse_fails("> 1 + 2");
    }

    #[test]
    fn test_if_expressions() {
        let input = "
            if 1 { 1 } else { 0 }
            if 2 { 2 }
            if (true) {}
        ";
        let expected = [
            "ExpressionStatement(IfExpression { condition: IntLiteral(1), consequence: \
            [ExpressionStatement(IntLiteral(1))], alternative: [ExpressionStatement(\
            IntLiteral(0))] })",
            "ExpressionStatement(IfExpression { condition: IntLiteral(2), consequence: \
            [ExpressionStatement(IntLiteral(2))], alternative: [] })",
            "ExpressionStatement(IfExpression { condition: Boolean(true), consequence: \
            [], alternative: [] })",
        ];
        assert_parse(input, &expected);

        assert_parse_fails("if true");
        assert_parse_fails("if { return 1; }");
        assert_parse_fails("if true {} else");
    }

    #[test]
    fn test_grouped_expression() {
        let input = "(2 + 3) * (5 + 7); (1 + (1 + (1 + 1)))";
        let expected = [
            "ExpressionStatement(InfixExpression(InfixExpression(IntLiteral(2), Plus, \
            IntLiteral(3)), Asterisk, InfixExpression(IntLiteral(5), Plus, IntLiteral(7))))",
            "ExpressionStatement(InfixExpression(IntLiteral(1), Plus, InfixExpression(\
            IntLiteral(1), Plus, InfixExpression(IntLiteral(1), Plus, IntLiteral(1)))))",
        ];
        assert_parse(input, &expected);

        assert_parse_fails("(1 + 1");
        assert_parse_fails("1 + 1)");
        assert_parse_fails(")(");
    }
}
