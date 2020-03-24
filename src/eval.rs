// @WIP: This whole module is a work in progress, expect function signatures to change
// @TODO: Add proper error handling. Currently, all evaluation functions panic instead of returning
// errors
use crate::ast::*;
use crate::object::*;
use crate::token::Token;

pub fn eval_expression(expression: Expression) -> Object {
    match expression {
        Expression::IntLiteral(i) => Object::Integer(i),
        Expression::Boolean(b) => Object::Boolean(b),
        Expression::PrefixExpression(tk, e) => {
            let right_side = eval_expression(*e);
            eval_prefix_operator(tk, right_side)
        }
        _ => panic!(),
    }
}

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

fn eval_prefix_operator(operator: Token, right: Object) -> Object {
    match (operator, right) {
        (Token::Minus, Object::Integer(i)) => Object::Integer(-i),
        (Token::Bang, obj) => Object::Boolean(!get_truth_value(obj)),
        _ => panic!(),
    }
}

fn get_truth_value(obj: Object) -> bool {
    match obj {
        Object::Boolean(b) => b,
        Object::Nil => false,
        // I am unsure if I want integer values to have a truth value or not. For now, I will stick
        // to the book, which specifies that they do
        Object::Integer(i) => i != 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Object::*;

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
        assert_eq!(eval_expression(Expression::IntLiteral(1)), Integer(1));
        assert_eq!(eval_expression(Expression::IntLiteral(2)), Integer(2));
        assert_eq!(eval_expression(Expression::IntLiteral(3)), Integer(3));
    }

    #[test]
    fn test_eval_bool_literal() {
        assert_eq!(eval_expression(Expression::Boolean(true)), Boolean(true));
        assert_eq!(eval_expression(Expression::Boolean(false)), Boolean(false));
    }

    #[test]
    fn test_eval_expression_statement() {
        let input = "42; true; 9";
        let expected = [Integer(42), Boolean(true), Integer(9)];
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
        let expected = [Integer(5), Boolean(false), Integer(3)];
        assert_eval(input, &expected);
    }

    #[test]
    fn test_bang_operator() {
        let input = "!!false; !0; !6";
        let expected = [Boolean(false), Boolean(true), Boolean(false)];
        assert_eval(input, &expected);
    }

    #[test]
    fn test_prefix_minus_operator() {
        let input = "-5; --42; -0";
        let expected = [Integer(-5), Integer(42), Integer(0)];
        assert_eval(input, &expected);
    }
}
