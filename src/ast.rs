use crate::token::*;

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
}

#[derive(Debug, Clone)]
pub enum Statement {
    Let(Box<LetStatement>),
    Return(Box<Expression>),
    ExpressionStatement(Box<Expression>),
    BlockStatement(Vec<Statement>),
}

#[derive(Debug, Clone)]
pub struct LetStatement {
    pub identifier: String,
    pub value: Expression,
}
