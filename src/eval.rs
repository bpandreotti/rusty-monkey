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
            eval_prefix_expression(tk, right_side)
        }
        Expression::InfixExpression(l, tk, r) => {
            let left_side = eval_expression(*l);
            let right_side = eval_expression(*r);
            eval_infix_expression(tk, left_side, right_side)
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

fn eval_prefix_expression(operator: Token, right: Object) -> Object {
    match (operator, right) {
        (Token::Minus, Object::Integer(i)) => Object::Integer(-i),
        (Token::Bang, obj) => Object::Boolean(!get_truth_value(obj)),
        _ => panic!(),
    }
}

fn eval_infix_expression(operator: Token, left: Object, right: Object) -> Object {
    match (left, right) {
        (Object::Integer(l), Object::Integer(r)) => eval_int_infix_expression(operator, l, r),
        _ => panic!(),
    }
}

fn eval_int_infix_expression(operator: Token, left: i64, right: i64) -> Object {
    match operator {
        // Arithmetic operators
        Token::Plus => Object::Integer(left + right),
        Token::Minus => Object::Integer(left - right),
        Token::Asterisk => Object::Integer(left * right),
        Token::Slash => Object::Integer(left / right),

        // Comparison operators
        Token::Equals => Object::Boolean(left == right),
        Token::NotEquals => Object::Boolean(left != right),
        Token::LessThan => Object::Boolean(left < right),
        Token::LessEq => Object::Boolean(left <= right),
        Token::GreaterThan => Object::Boolean(left > right),
        Token::GreaterEq => Object::Boolean(left >= right),

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

        // Parse program into vector of statements
        let parsed = Parser::new(Lexer::new(input.into()))
            .parse_program()
            .expect("Parser error during test");

        assert_eq!(parsed.len(), expected.len());

        // Eval program statements and compare with expected
        parsed
            .into_iter()
            .map(eval_statement)
            .zip(expected)
            .for_each(|(got, exp)| assert_eq!(&got, exp));
    }

    #[test]
    fn test_eval_int_expression() {
        let input = "
            5;
            -10;
            --42;
            -0;
            2 + 2;
            1 * 2 + 3;
            1 + 2 * 3;
            (1 + 1) * (2 + 2);
            66 / (2 * 3 + 5);
        ";
        let expected = [
            Integer(5),
            Integer(-10),
            Integer(42),
            Integer(0),
            Integer(4),
            Integer(5),
            Integer(7),
            Integer(8),
            Integer(6),
        ];
        assert_eval(input, &expected);
    }

    #[test]
    fn test_eval_bool_expression() {
        let input = "
            false;
            !true;
            !!true;
            1 < 2;
            2 <= 0;
            1 > 2;
            2 >= 0;
            0 == 0;
            1 != 0;
        ";
        let expected = [
            Boolean(false),
            Boolean(false),
            Boolean(true),
            Boolean(true),
            Boolean(false),
            Boolean(false),
            Boolean(true),
            Boolean(true),
            Boolean(true),
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
        let expected = [Integer(5), Boolean(false), Integer(3)];
        assert_eval(input, &expected);
    }
}
