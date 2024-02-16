use std::fmt::Display;

#[derive(Debug, PartialEq)]
pub enum Statement {
    EmptyStatement,
    LetStatement { name: String, value: Expression },
    ReturnStatement(Expression),
    ExpressionStatement(Expression),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    EmptyExpression,
    Identifier(String),
    IntegerLiteral(i64),
    PrefixExpression {
        operator: String,
        right: Box<Expression>,
    },
    InfixExpression {
        left: Box<Expression>,
        operator: String,
        right: Box<Expression>,
    },
}

#[derive(Debug, PartialEq)]
pub struct Program {
    pub statements: Vec<Statement>,
}

impl Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::EmptyExpression => write!(f, ""),
            Expression::Identifier(ident) => write!(f, "{}", ident),
            Expression::IntegerLiteral(int) => write!(f, "{}", int),
            Expression::PrefixExpression { operator, right } => {
                write!(f, "({}{})", operator, right)
            }
            Expression::InfixExpression {
                left,
                operator,
                right,
            } => write!(f, "({} {} {})", left, operator, right),
        }
    }
}

impl Display for Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Statement::EmptyStatement => write!(f, ""),
            Statement::LetStatement { name, value } => write!(f, "let {} = {};", name, value),
            Statement::ReturnStatement(expr) => write!(f, "return {};", expr),
            Statement::ExpressionStatement(expr) => write!(f, "{}", expr),
        }
    }
}

impl Display for Program {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let result = self
            .statements
            .iter()
            .map(|s| format!("{}", s))
            .collect::<Vec<String>>()
            .join("\n");
        write!(f, "{}", result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string() {
        let program = Program {
            statements: vec![
                Statement::LetStatement {
                    name: "myVar".to_string(),
                    value: Expression::Identifier("anotherVar".to_string()),
                },
                Statement::ReturnStatement(Expression::IntegerLiteral(5)),
            ],
        };
        assert_eq!(format!("{}", program), "let myVar = anotherVar;\nreturn 5;");
    }
}
