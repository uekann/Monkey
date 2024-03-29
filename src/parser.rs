use crate::ast::{Expression, Program, Statement};
use crate::lexer::Lexer;
use crate::token::{Token, TokenType};
use anyhow::{ensure, Result};

#[derive(Debug, PartialEq, PartialOrd)]
enum Precedence {
    LOWEST,
    EQUALS,      // ==
    LESSGREATER, // > or <
    SUM,         // +
    PRODUCT,     // *
    PREFIX,      // -X or !X
    CALL,        // myFunction(X)
}

impl Precedence {
    fn from_token_type(t: TokenType) -> Precedence {
        match t {
            TokenType::EQ => Precedence::EQUALS,
            TokenType::NOT_EQ => Precedence::EQUALS,
            TokenType::LT => Precedence::LESSGREATER,
            TokenType::GT => Precedence::LESSGREATER,
            TokenType::PLUS => Precedence::SUM,
            TokenType::MINUS => Precedence::SUM,
            TokenType::SLASH => Precedence::PRODUCT,
            TokenType::ASTERISK => Precedence::PRODUCT,
            TokenType::LPAREN => Precedence::CALL,
            _ => Precedence::LOWEST,
        }
    }
}

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    cur_token: Token,
    peek_token: Token,
}

impl<'a> Parser<'a> {
    pub fn new(lexer: Lexer<'a>) -> Parser<'a> {
        let mut p = Parser {
            lexer: lexer,
            cur_token: Token {
                token_type: TokenType::EOF,
                literal: String::new(),
            },
            peek_token: Token {
                token_type: TokenType::EOF,
                literal: String::new(),
            },
        };

        p.next_token();
        p.next_token();
        p
    }

    pub fn next_token(&mut self) {
        self.cur_token = self.peek_token.clone();
        self.peek_token = self.lexer.next_token();
    }

    fn expect_peek(&mut self, t: TokenType) -> bool {
        if self.peek_token.token_type == t {
            self.next_token();
            true
        } else {
            false
        }
    }

    fn cur_precedence(&self) -> Precedence {
        Precedence::from_token_type(self.cur_token.token_type)
    }

    fn get_prefix_parse_fn(
        &self,
        t: TokenType,
    ) -> Option<fn(&mut Parser<'a>) -> Result<Expression>> {
        match t {
            TokenType::IDENT => Some(Parser::parse_identifier),
            TokenType::INT => Some(Parser::parse_integer_literal),
            TokenType::TRUE => Some(Parser::parse_boolean),
            TokenType::FALSE => Some(Parser::parse_boolean),
            TokenType::BANG => Some(Parser::parse_prefix_expression),
            TokenType::MINUS => Some(Parser::parse_prefix_expression),
            TokenType::LPAREN => Some(Parser::parse_grouped_expression),
            TokenType::IF => Some(Parser::parse_if_expression),
            TokenType::FUNCTION => Some(Parser::parse_function_literal),
            _ => None,
        }
    }

    fn get_infix_parse_fn(
        &self,
        t: TokenType,
    ) -> Option<fn(&mut Parser<'a>, Expression) -> Result<Expression>> {
        match t {
            TokenType::PLUS => Some(Parser::parse_infix_expression),
            TokenType::MINUS => Some(Parser::parse_infix_expression),
            TokenType::SLASH => Some(Parser::parse_infix_expression),
            TokenType::ASTERISK => Some(Parser::parse_infix_expression),
            TokenType::EQ => Some(Parser::parse_infix_expression),
            TokenType::NOT_EQ => Some(Parser::parse_infix_expression),
            TokenType::LT => Some(Parser::parse_infix_expression),
            TokenType::GT => Some(Parser::parse_infix_expression),
            TokenType::LPAREN => Some(Parser::parse_call_expression),
            _ => None,
        }
    }

    pub fn parse_program(&mut self) -> Result<Program> {
        let mut program = Program {
            statements: Vec::new(),
        };

        while self.cur_token.token_type != TokenType::EOF {
            let stmt = self.parse_statement()?;
            if stmt != Statement::EmptyStatement {
                program.statements.push(stmt);
            }
            self.next_token();
        }

        Ok(program)
    }

    fn parse_statement(&mut self) -> Result<Statement> {
        match self.cur_token.token_type {
            TokenType::LET => self.parse_let_statement(),
            TokenType::RETURN => self.parse_return_statement(),
            TokenType::LBRACE => self.parse_block_statement(),
            _ => self.parse_expression_statement(),
        }
    }

    fn parse_let_statement(&mut self) -> Result<Statement> {
        ensure!(
            self.expect_peek(TokenType::IDENT),
            "expected next token to be IDENT, got {:?} instead",
            self.peek_token.token_type
        );
        let name = self.cur_token.literal.to_string();

        ensure!(
            self.expect_peek(TokenType::ASSIGN),
            "expected next token to be ASSIGN, got {:?} instead",
            self.peek_token.token_type
        );

        self.next_token();

        let value = self.parse_expression(Precedence::LOWEST)?;
        ensure!(
            self.expect_peek(TokenType::SEMICOLON),
            "expected next token to be SEMICOLON, got {:?} instead",
            self.peek_token.token_type
        );
        Ok(Statement::LetStatement { name, value })
    }

    fn parse_return_statement(&mut self) -> Result<Statement> {
        self.next_token();
        let value = self.parse_expression(Precedence::LOWEST)?;
        ensure!(
            self.expect_peek(TokenType::SEMICOLON),
            "expected next token to be SEMICOLON, got {:?} instead",
            self.peek_token.token_type
        );
        Ok(Statement::ReturnStatement(value))
    }

    fn parse_block_statement(&mut self) -> Result<Statement> {
        self.next_token();
        let mut statements = Vec::new();
        while self.cur_token.token_type != TokenType::RBRACE
            && self.cur_token.token_type != TokenType::EOF
        {
            let stmt = self.parse_statement()?;
            if stmt != Statement::EmptyStatement {
                statements.push(stmt);
            }
            self.next_token();
        }
        Ok(Statement::BlockStatement { statements })
    }

    fn parse_expression_statement(&mut self) -> Result<Statement> {
        let expression = self.parse_expression(Precedence::LOWEST)?;
        if self.peek_token.token_type == TokenType::SEMICOLON {
            self.next_token();
        }
        Ok(Statement::ExpressionStatement(expression))
    }

    fn parse_expression(&mut self, precedence: Precedence) -> Result<Expression> {
        let prefix = self.parse_prefix()?;
        let mut left = prefix;

        while self.peek_token.token_type != TokenType::SEMICOLON
            && precedence < Precedence::from_token_type(self.peek_token.token_type)
        {
            self.next_token();
            left = self.parse_infix(left)?;
        }

        Ok(left)
    }

    fn parse_prefix(&mut self) -> Result<Expression> {
        let prefix_fn = self.get_prefix_parse_fn(self.cur_token.token_type);
        ensure!(
            prefix_fn.is_some(),
            "no prefix parse function for {:?} found",
            self.cur_token.token_type
        );
        let prefix_fn = prefix_fn.unwrap();
        prefix_fn(self)
    }

    fn parse_infix(&mut self, left: Expression) -> Result<Expression> {
        let token_type = self.cur_token.token_type;
        let infix_fn = self.get_infix_parse_fn(token_type);
        if infix_fn.is_none() {
            return Ok(left);
        }
        let infix_fn = infix_fn.unwrap().clone();
        infix_fn(self, left)
    }

    fn parse_identifier(&mut self) -> Result<Expression> {
        Ok(Expression::Identifier(self.cur_token.literal.to_string()))
    }

    fn parse_integer_literal(&mut self) -> Result<Expression> {
        let value = self.cur_token.literal.parse::<i64>()?;
        Ok(Expression::IntegerLiteral(value))
    }

    fn parse_boolean(&mut self) -> Result<Expression> {
        Ok(Expression::Boolean(
            self.cur_token.token_type == TokenType::TRUE,
        ))
    }

    fn parse_prefix_expression(&mut self) -> Result<Expression> {
        let operator = self.cur_token.literal.to_string();
        ensure!(
            self.cur_token.token_type == TokenType::BANG
                || self.cur_token.token_type == TokenType::MINUS,
            "expected token to be BANG or MINUS, got {:?} instead",
            self.cur_token.token_type
        );
        self.next_token();
        let right = self.parse_expression(Precedence::PREFIX)?;
        Ok(Expression::PrefixExpression {
            operator,
            right: Box::new(right),
        })
    }

    fn parse_grouped_expression(&mut self) -> Result<Expression> {
        self.next_token();
        let expression = self.parse_expression(Precedence::LOWEST)?;
        ensure!(
            self.expect_peek(TokenType::RPAREN),
            "expected next token to be RPAREN, got {:?} instead",
            self.peek_token.token_type
        );
        Ok(expression)
    }

    fn parse_if_expression(&mut self) -> Result<Expression> {
        ensure!(
            self.expect_peek(TokenType::LPAREN),
            "expected next token to be LPAREN, got {:?} instead",
            self.peek_token.token_type
        );
        self.next_token();
        let condition = self.parse_expression(Precedence::LOWEST)?;
        ensure!(
            self.expect_peek(TokenType::RPAREN),
            "expected next token to be RPAREN, got {:?} instead",
            self.peek_token.token_type
        );
        ensure!(
            self.expect_peek(TokenType::LBRACE),
            "expected next token to be LBRACE, got {:?} instead",
            self.peek_token.token_type
        );
        let consequence = self.parse_block_statement()?;
        let alternative = if self.peek_token.token_type == TokenType::ELSE {
            self.next_token();
            ensure!(
                self.expect_peek(TokenType::LBRACE),
                "expected next token to be LBRACE, got {:?} instead",
                self.peek_token.token_type
            );
            Some(Box::new(self.parse_block_statement()?))
        } else {
            None
        };
        Ok(Expression::IfExpression {
            condition: Box::new(condition),
            consequence: Box::new(consequence),
            alternative: alternative,
        })
    }

    fn parse_function_parameters(&mut self) -> Result<Vec<Expression>> {
        let mut identifiers = Vec::new();
        if self.peek_token.token_type == TokenType::RPAREN {
            self.next_token();
            return Ok(identifiers);
        }
        self.next_token();
        identifiers.push(Expression::Identifier(self.cur_token.literal.to_string()));
        while self.peek_token.token_type == TokenType::COMMA {
            self.next_token();
            self.next_token();
            identifiers.push(Expression::Identifier(self.cur_token.literal.to_string()));
        }
        ensure!(
            self.expect_peek(TokenType::RPAREN),
            "expected next token to be RPAREN, got {:?} instead",
            self.peek_token.token_type
        );
        Ok(identifiers)
    }

    fn parse_function_literal(&mut self) -> Result<Expression> {
        ensure!(
            self.expect_peek(TokenType::LPAREN),
            "expected next token to be LPAREN, got {:?} instead",
            self.peek_token.token_type
        );
        let parameters = self.parse_function_parameters()?;
        ensure!(
            self.expect_peek(TokenType::LBRACE),
            "expected next token to be LBRACE, got {:?} instead",
            self.peek_token.token_type
        );
        let body = self.parse_block_statement()?;
        Ok(Expression::FunctionLiteral {
            parameters,
            body: Box::new(body),
        })
    }

    fn parse_infix_expression(&mut self, left: Expression) -> Result<Expression> {
        let operator = self.cur_token.literal.to_string();
        let precedence = self.cur_precedence();
        self.next_token();
        let right = self.parse_expression(precedence)?;
        Ok(Expression::InfixExpression {
            left: Box::new(left),
            operator,
            right: Box::new(right),
        })
    }

    fn parse_call_arguments(&mut self) -> Result<Vec<Expression>> {
        let mut args = Vec::new();
        if self.peek_token.token_type == TokenType::RPAREN {
            self.next_token();
            return Ok(args);
        }
        self.next_token();
        args.push(self.parse_expression(Precedence::LOWEST)?);
        while self.peek_token.token_type == TokenType::COMMA {
            self.next_token();
            self.next_token();
            args.push(self.parse_expression(Precedence::LOWEST)?);
        }
        ensure!(
            self.expect_peek(TokenType::RPAREN),
            "expected next token to be RPAREN, got {:?} instead",
            self.peek_token.token_type
        );
        Ok(args)
    }

    fn parse_call_expression(&mut self, function: Expression) -> Result<Expression> {
        let arguments = self.parse_call_arguments()?;
        Ok(Expression::CallExpression {
            function: Box::new(function),
            arguments,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    #[test]
    fn test_let_statements() {
        let input = r#"
let x = 5;
let y = 10;
let foobar = 838383;
let t = true;
let f = false;
"#;
        let l = Lexer::new(input);
        let mut p = Parser::new(l);
        let program = p.parse_program().unwrap();
        assert_eq!(program.statements.len(), 5);

        let tests = vec![
            Statement::LetStatement {
                name: "x".to_string(),
                value: Expression::IntegerLiteral(5),
            },
            Statement::LetStatement {
                name: "y".to_string(),
                value: Expression::IntegerLiteral(10),
            },
            Statement::LetStatement {
                name: "foobar".to_string(),
                value: Expression::IntegerLiteral(838383),
            },
            Statement::LetStatement {
                name: "t".to_string(),
                value: Expression::Boolean(true),
            },
            Statement::LetStatement {
                name: "f".to_string(),
                value: Expression::Boolean(false),
            },
        ];
        for (i, tt) in tests.iter().enumerate() {
            assert_eq!(&program.statements[i], tt);
        }
    }

    #[test]
    fn test_let_statements_error() {
        let input = r#"
let x 5;
let = 10;
let 838383;
"#;
        let l = Lexer::new(input);
        let mut p = Parser::new(l);
        let program = p.parse_program();
        assert!(program.is_err());
    }

    #[test]
    fn test_return_statements() {
        let input = r#"
return 5;
return 10;
return 838383;
"#;
        let l = Lexer::new(input);
        let mut p = Parser::new(l);
        let program = p.parse_program().unwrap();
        assert_eq!(program.statements.len(), 3);

        let tests = vec![
            Statement::ReturnStatement(Expression::IntegerLiteral(5)),
            Statement::ReturnStatement(Expression::IntegerLiteral(10)),
            Statement::ReturnStatement(Expression::IntegerLiteral(838383)),
        ];
        for (i, tt) in tests.iter().enumerate() {
            assert_eq!(&program.statements[i], tt);
        }
    }

    #[test]
    fn test_infix_expression() {
        let input = r#"
5 + 5;
5 - 5;
5 * 5;
5 / 5;
5 > 5;
5 < 5;
5 == 5;
5 != 5;
"#;
        let l = Lexer::new(input);
        let mut p = Parser::new(l);
        let program = p.parse_program().unwrap();
        assert_eq!(program.statements.len(), 8);

        let tests = vec![
            Expression::InfixExpression {
                left: Box::new(Expression::IntegerLiteral(5)),
                operator: "+".to_string(),
                right: Box::new(Expression::IntegerLiteral(5)),
            },
            Expression::InfixExpression {
                left: Box::new(Expression::IntegerLiteral(5)),
                operator: "-".to_string(),
                right: Box::new(Expression::IntegerLiteral(5)),
            },
            Expression::InfixExpression {
                left: Box::new(Expression::IntegerLiteral(5)),
                operator: "*".to_string(),
                right: Box::new(Expression::IntegerLiteral(5)),
            },
            Expression::InfixExpression {
                left: Box::new(Expression::IntegerLiteral(5)),
                operator: "/".to_string(),
                right: Box::new(Expression::IntegerLiteral(5)),
            },
            Expression::InfixExpression {
                left: Box::new(Expression::IntegerLiteral(5)),
                operator: ">".to_string(),
                right: Box::new(Expression::IntegerLiteral(5)),
            },
            Expression::InfixExpression {
                left: Box::new(Expression::IntegerLiteral(5)),
                operator: "<".to_string(),
                right: Box::new(Expression::IntegerLiteral(5)),
            },
            Expression::InfixExpression {
                left: Box::new(Expression::IntegerLiteral(5)),
                operator: "==".to_string(),
                right: Box::new(Expression::IntegerLiteral(5)),
            },
            Expression::InfixExpression {
                left: Box::new(Expression::IntegerLiteral(5)),
                operator: "!=".to_string(),
                right: Box::new(Expression::IntegerLiteral(5)),
            },
        ];
        for (i, tt) in tests.iter().enumerate() {
            assert_eq!(
                &program.statements[i],
                &Statement::ExpressionStatement(tt.clone())
            );
        }
    }

    #[test]
    fn test_prefix_expression() {
        let input = r#"
!5;
-15;
"#;
        let l = Lexer::new(input);
        let mut p = Parser::new(l);
        let program = p.parse_program().unwrap();
        assert_eq!(program.statements.len(), 2);

        let tests = vec![
            Expression::PrefixExpression {
                operator: "!".to_string(),
                right: Box::new(Expression::IntegerLiteral(5)),
            },
            Expression::PrefixExpression {
                operator: "-".to_string(),
                right: Box::new(Expression::IntegerLiteral(15)),
            },
        ];
        for (i, tt) in tests.iter().enumerate() {
            assert_eq!(
                &program.statements[i],
                &Statement::ExpressionStatement(tt.clone())
            );
        }
    }

    #[test]
    fn test_operator_precedence_parsing() {
        let tests = vec![
            ("!-a", "(!(-a))"),
            ("a + b", "(a + b)"),
            ("a * b", "(a * b)"),
            ("a + b * c", "(a + (b * c))"),
            ("a + b / c", "(a + (b / c))"),
            ("a + b * c + d / e - f", "(((a + (b * c)) + (d / e)) - f)"),
            ("3 + 4; -5 * 5", "(3 + 4)\n((-5) * 5)"),
            ("5 > 4 == 3 < 4", "((5 > 4) == (3 < 4))"),
            ("5 < 4 != 3 > 4", "((5 < 4) != (3 > 4))"),
            (
                "3 + 4 * 5 == 3 * 1 + 4 * 5",
                "((3 + (4 * 5)) == ((3 * 1) + (4 * 5)))",
            ),
        ];
        for (input, expected) in tests {
            let l = Lexer::new(input);
            let mut p = Parser::new(l);
            let program = p.parse_program().unwrap();
            assert_eq!(format!("{}", program), expected);
        }
    }

    #[test]
    fn test_if_expression() {
        let input = r#"
if (x < y) { x }
"#;
        let l = Lexer::new(input);
        let mut p = Parser::new(l);
        let program = p.parse_program().unwrap();
        assert_eq!(program.statements.len(), 1);

        let tests = vec![Statement::ExpressionStatement(Expression::IfExpression {
            condition: Box::new(Expression::InfixExpression {
                left: Box::new(Expression::Identifier("x".to_string())),
                operator: "<".to_string(),
                right: Box::new(Expression::Identifier("y".to_string())),
            }),
            consequence: Box::new(Statement::BlockStatement {
                statements: vec![Statement::ExpressionStatement(Expression::Identifier(
                    "x".to_string(),
                ))],
            }),
            alternative: None,
        })];
        for (i, tt) in tests.iter().enumerate() {
            assert_eq!(&program.statements[i], tt);
        }
    }

    #[test]
    fn test_if_else_expression() {
        let input = r#"
if (x < y) { x } else { y }
"#;
        let l = Lexer::new(input);
        let mut p = Parser::new(l);
        let program = p.parse_program().unwrap();
        assert_eq!(program.statements.len(), 1);

        let tests = vec![Statement::ExpressionStatement(Expression::IfExpression {
            condition: Box::new(Expression::InfixExpression {
                left: Box::new(Expression::Identifier("x".to_string())),
                operator: "<".to_string(),
                right: Box::new(Expression::Identifier("y".to_string())),
            }),
            consequence: Box::new(Statement::BlockStatement {
                statements: vec![Statement::ExpressionStatement(Expression::Identifier(
                    "x".to_string(),
                ))],
            }),
            alternative: Some(Box::new(Statement::BlockStatement {
                statements: vec![Statement::ExpressionStatement(Expression::Identifier(
                    "y".to_string(),
                ))],
            })),
        })];
        for (i, tt) in tests.iter().enumerate() {
            assert_eq!(&program.statements[i], tt);
        }
    }

    #[test]
    fn test_function_literal_parsing() {
        let input = r#"
fn(x, y) { x + y; }
"#;
        let l = Lexer::new(input);
        let mut p = Parser::new(l);
        let program = p.parse_program().unwrap();
        assert_eq!(program.statements.len(), 1);

        let tests = vec![Statement::ExpressionStatement(
            Expression::FunctionLiteral {
                parameters: vec![
                    Expression::Identifier("x".to_string()),
                    Expression::Identifier("y".to_string()),
                ],
                body: Box::new(Statement::BlockStatement {
                    statements: vec![Statement::ExpressionStatement(
                        Expression::InfixExpression {
                            left: Box::new(Expression::Identifier("x".to_string())),
                            operator: "+".to_string(),
                            right: Box::new(Expression::Identifier("y".to_string())),
                        },
                    )],
                }),
            },
        )];
        for (i, tt) in tests.iter().enumerate() {
            assert_eq!(&program.statements[i], tt);
        }
    }

    #[test]
    fn test_function_parameter_parsing() {
        let tests = vec![
            ("fn() {};", vec![]),
            ("fn(x) {};", vec!["x"]),
            ("fn(x, y, z) {};", vec!["x", "y", "z"]),
        ];
        for (input, expected) in tests {
            let l = Lexer::new(input);
            let mut p = Parser::new(l);
            let program = p.parse_program().unwrap();
            let stmt = program.statements.first().unwrap();
            let expected = expected
                .iter()
                .map(|s| Expression::Identifier(s.to_string()))
                .collect::<Vec<Expression>>();
            assert_eq!(
                &Statement::ExpressionStatement(Expression::FunctionLiteral {
                    parameters: expected,
                    body: Box::new(Statement::BlockStatement { statements: vec![] })
                }),
                stmt
            );
        }
    }

    #[test]
    fn test_call_expression_parsing() {
        let input = r#"
add(1, 2 * 3, 4 + 5);
"#;
        let l = Lexer::new(input);
        let mut p = Parser::new(l);
        let program = p.parse_program().unwrap();
        assert_eq!(program.statements.len(), 1);

        let tests = vec![Statement::ExpressionStatement(Expression::CallExpression {
            function: Box::new(Expression::Identifier("add".to_string())),
            arguments: vec![
                Expression::IntegerLiteral(1),
                Expression::InfixExpression {
                    left: Box::new(Expression::IntegerLiteral(2)),
                    operator: "*".to_string(),
                    right: Box::new(Expression::IntegerLiteral(3)),
                },
                Expression::InfixExpression {
                    left: Box::new(Expression::IntegerLiteral(4)),
                    operator: "+".to_string(),
                    right: Box::new(Expression::IntegerLiteral(5)),
                },
            ],
        })];
        for (i, tt) in tests.iter().enumerate() {
            assert_eq!(&program.statements[i], tt);
        }
    }
}
