#[derive(Debug)]
pub enum Expression {
    Identifier(String),
    IntLiteral(i64),
    PrefixBang(Box<Expression>),
    PrefixMinus(Box<Expression>),
    Nil,
}

#[derive(Debug)]
pub enum Statement {
    Let(Box<LetStatement>),
    Return(Box<Expression>),
    ExpressionStatement(Box<Expression>),
}

#[derive(Debug)]
pub struct LetStatement {
    pub identifier: String,
    pub value: Expression,
}