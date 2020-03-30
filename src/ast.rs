use crate::token::*;

pub type LetStatement = (String, Expression);

#[derive(Debug, Clone)]
pub enum Expression {
    Identifier(String),
    IntLiteral(i64),
    StringLiteral(String),
    Boolean(bool),
    ArrayLiteral(Vec<Expression>),
    HashLiteral(Vec<(Expression, Expression)>),
    IndexExpression(Box<Expression>, Box<Expression>),
    PrefixExpression(Token, Box<Expression>),
    InfixExpression(Box<Expression>, Token, Box<Expression>),
    BlockExpression(Vec<Statement>),
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
