use std::collections::HashMap;
use std::vec;

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
    prefix_parse_fns: HashMap<TokenType, fn(&mut Parser<'a>) -> Result<Expression>>,
    infix_parse_fns: HashMap<TokenType, fn(&mut Parser<'a>, Expression) -> Result<Expression>>,
}

impl<'a> Parser<'a> {
    pub fn new(lexer: Lexer<'a>) -> Parser<'a> {
        let prefix_parse_funs_vec: Vec<(TokenType, fn(&mut Parser<'a>) -> Result<Expression>)> = vec![
            (TokenType::IDENT, Parser::parse_identifier),
            (TokenType::INT, Parser::parse_integer_literal),
            (TokenType::BANG, Parser::parse_prefix_expression),
            (TokenType::MINUS, Parser::parse_prefix_expression),
            (TokenType::LPAREN, Parser::parse_grouped_expression),
        ];

        let infix_parse_funs_vec: Vec<(
            TokenType,
            fn(&mut Parser<'a>, Expression) -> Result<Expression>,
        )> = vec![
            (TokenType::PLUS, Parser::parse_infix_expression),
            (TokenType::MINUS, Parser::parse_infix_expression),
            (TokenType::SLASH, Parser::parse_infix_expression),
            (TokenType::ASTERISK, Parser::parse_infix_expression),
            (TokenType::EQ, Parser::parse_infix_expression),
            (TokenType::NOT_EQ, Parser::parse_infix_expression),
            (TokenType::LT, Parser::parse_infix_expression),
            (TokenType::GT, Parser::parse_infix_expression),
        ];

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
            prefix_parse_fns: prefix_parse_funs_vec.into_iter().collect(),
            infix_parse_fns: infix_parse_funs_vec.into_iter().collect(),
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
        let prefix_fn = self.prefix_parse_fns.get(&self.cur_token.token_type);
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
        let infix_fn = self.infix_parse_fns.get(&token_type);
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
"#;
        let l = Lexer::new(input);
        let mut p = Parser::new(l);
        let program = p.parse_program().unwrap();
        assert_eq!(program.statements.len(), 3);

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
}
