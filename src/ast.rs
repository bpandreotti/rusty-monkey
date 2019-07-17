use crate::token::*;

#[derive(Debug)]
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
    Nil,
}

#[derive(Debug)]
pub enum Statement {
    Let(Box<LetStatement>),
    Return(Box<Expression>),
    ExpressionStatement(Box<Expression>),
    BlockStatement(Vec<Statement>),
}

#[derive(Debug)]
pub struct LetStatement {
    pub identifier: String,
    pub value: Expression,
}
