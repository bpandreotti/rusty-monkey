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
pub fn eval_statement(statement: Statement) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eval_int_literal() {
        assert_eq!(eval_expression(Expression::IntLiteral(1)), Object::Integer(1));
        assert_eq!(eval_expression(Expression::IntLiteral(2)), Object::Integer(2));
        assert_eq!(eval_expression(Expression::IntLiteral(3)), Object::Integer(3));
    }

    #[test]
    fn test_eval_bool_literal() {
        assert_eq!(
            eval_expression(Expression::Boolean(true)),
            Object::Boolean(true)
        );
        assert_eq!(
            eval_expression(Expression::Boolean(false)),
            Object::Boolean(false)
        );
    }
}
