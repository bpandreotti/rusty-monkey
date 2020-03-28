// @TODO: Consider implementing block expressions. Block expressions are like block statements, but
// they can can be used in contexts where an expression is expected. Example:
//     let a = {
//         let b = 20;
//         b * (b - 1)
//     };
// Currently, this behaviour can be achieved by using an if expression:
//     let a = if true {
//         let b = 20;
//         b * (b - 1)
//     };
// Once block expressions are working, consider implementing block statements as expression
// statements wrapping a block expresison. This might conflict with parsing Hashes.

use crate::token::*;

pub type LetStatement = (String, Expression);

#[derive(Debug, Clone)]
pub enum Expression {
    Identifier(String),
    IntLiteral(i64),
    Boolean(bool),
    PrefixExpression(Token, Box<Expression>),
    InfixExpression(Box<Expression>, Token, Box<Expression>),
    IfExpression {
        condition: Box<Expression>,
        consequence: Vec<Statement>,
        alternative: Vec<Statement>,
    },
    FunctionLiteral {
        parameters: Vec<String>,
        body: Vec<Statement>,
    },
    CallExpression {
        function: Box<Expression>,
        arguments: Vec<Expression>,
    },
    Nil,
}

#[derive(Debug, Clone)]
pub enum Statement {
    Let(Box<LetStatement>),
    Return(Box<Expression>),
    ExpressionStatement(Box<Expression>),
    BlockStatement(Vec<Statement>),
}
