#[derive(Debug)]
pub enum Expression {
    Nil,
}

#[derive(Debug)]
pub enum Statement {
    LetStatement(Box<LetStatement>),
}

#[derive(Debug)]
pub struct LetStatement {
    pub identifier: String,
    pub value: Expression,
}
