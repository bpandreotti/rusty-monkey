#[derive(Debug)]
pub enum Expression {
    Nil,
}

#[derive(Debug)]
pub enum Statement {
    Let(Box<LetStatement>),
    Return(Box<Expression>),
}

#[derive(Debug)]
pub struct LetStatement {
    pub identifier: String,
    pub value: Expression,
}
