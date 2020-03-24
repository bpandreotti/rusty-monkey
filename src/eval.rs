use crate::ast::*;
use crate::object::*;

// @WIP
pub fn eval_expression(expression: Expression) -> Object {
    match expression {
        Expression::IntLiteral(i) => Object::Integer(i),
        Expression::Boolean(b) => Object::Boolean(b),
        _ => panic!(),
    }
}

// @WIP
pub fn eval_statement(statement: Statement) -> Object {
    match statement {
        Statement::ExpressionStatement(exp) => eval_expression(*exp),
        Statement::BlockStatement(block) => {
            let mut last = Object::Nil;
            for s in block {
                last = eval_statement(s);
            }
            last
        }
        _ => panic!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_eval(input: &str, expected: &[Object]) {
        use crate::lexer::Lexer;
        use crate::parser::Parser;

        Parser::new(Lexer::new(input.into()))
            .parse_program() // Parse program into vector of statements
            .expect("Parser error during test")
            .into_iter()
            .map(eval_statement) // Eval program statements
            .zip(expected)
            .for_each(|(got, exp)| assert_eq!(&got, exp)); // Compare with expected
    }


    #[test]
    fn test_eval_int_literal() {
        assert_eq!(eval_expression(Expression::IntLiteral(1)), Object::Integer(1));
        assert_eq!(eval_expression(Expression::IntLiteral(2)), Object::Integer(2));
        assert_eq!(eval_expression(Expression::IntLiteral(3)), Object::Integer(3));
    }

    #[test]
    fn test_eval_bool_literal() {
        assert_eq!(eval_expression(Expression::Boolean(true)), Object::Boolean(true));
        assert_eq!(eval_expression(Expression::Boolean(false)), Object::Boolean(false));
    }

    #[test]
    fn test_eval_expression_statement() {
        let input = "42; true; 9";
        let expected = [
            Object::Integer(42),
            Object::Boolean(true),
            Object::Integer(9),
        ];

        assert_eval(input, &expected);
    }

    #[test]
    fn test_eval_block_statement() {
        let input = "
            { 5 }
            { 2; false }
            {
                { true; 3; }
            }
        ";

        let expected = [
            Object::Integer(5),
            Object::Boolean(false),
            Object::Integer(3),
        ];

        assert_eval(input, &expected);
    }
}
