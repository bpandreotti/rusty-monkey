#[derive(Debug, PartialEq)]
pub enum Object {
    Integer(i64),
    Boolean(bool),
    Nil, // @TODO: Consider implementing Nil/Null type
}
