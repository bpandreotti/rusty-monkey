use crate::token::*;

use std::fmt;

#[derive(Clone)]
pub struct NodeExpression {
    pub position: (usize, usize),
    pub expression: Expression,
}

impl fmt::Debug for NodeExpression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.expression)
    }
}

#[derive(Debug, Clone)]
pub enum Expression {
    Identifier(String),
    IntLiteral(i64),
    StringLiteral(String),
    Boolean(bool),
    ArrayLiteral(Vec<NodeExpression>),
    HashLiteral(Vec<(NodeExpression, NodeExpression)>),
    IndexExpression(Box<NodeExpression>, Box<NodeExpression>),
    PrefixExpression(Token, Box<NodeExpression>),
    InfixExpression(Box<NodeExpression>, Token, Box<NodeExpression>),
    BlockExpression(Vec<NodeStatement>),
    IfExpression {
        condition: Box<NodeExpression>,
        consequence: Vec<NodeStatement>,
        alternative: Vec<NodeStatement>,
    },
    FunctionLiteral {
        parameters: Vec<String>,
        body: Vec<NodeStatement>,
    },
    CallExpression {
        function: Box<NodeExpression>,
        arguments: Vec<NodeExpression>,
    },
    Nil,
}

#[derive(Clone)]
pub struct NodeStatement {
    pub position: (usize, usize),
    pub statement: Statement,
}

impl fmt::Debug for NodeStatement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.statement)
    }
}
pub type LetStatement = (String, NodeExpression);

#[derive(Debug, Clone)]
pub enum Statement {
    Let(Box<LetStatement>),
    Return(Box<NodeExpression>),
    ExpressionStatement(Box<NodeExpression>),
}
