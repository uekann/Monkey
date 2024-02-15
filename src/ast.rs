#[derive(Debug, PartialEq)]
pub enum Statement {
    EmptyStatement,
    LetStatement { name: String, value: Expression },
}

#[derive(Debug, PartialEq)]
pub enum Expression {
    EmptyExpression,
    Identifier(String),
    IntegerLiteral(i64),
}

#[derive(Debug, PartialEq)]
pub struct Program {
    pub statements: Vec<Statement>,
}
