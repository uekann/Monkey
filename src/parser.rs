use crate::ast::{Expression, Program, Statement};
use crate::lexer::Lexer;
use crate::token::{Token, TokenType};
use anyhow::{ensure, Result};

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    cur_token: Token<'a>,
    peek_token: Token<'a>,
}

impl<'a> Parser<'a> {
    pub fn new(lexer: Lexer<'a>) -> Parser<'a> {
        let mut p = Parser {
            lexer: lexer,
            cur_token: Token {
                token_type: TokenType::EOF,
                literal: "",
            },
            peek_token: Token {
                token_type: TokenType::EOF,
                literal: "",
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
            _ => Ok(Statement::EmptyStatement),
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

        let value = self.parse_expression()?;
        Ok(Statement::LetStatement { name, value })
    }

    fn parse_expression(&mut self) -> Result<Expression> {
        match self.cur_token.token_type {
            TokenType::IDENT => Ok(Expression::Identifier(self.cur_token.literal.to_string())),
            TokenType::INT => {
                let value = self.cur_token.literal.parse::<i64>().unwrap();
                Ok(Expression::IntegerLiteral(value))
            }
            _ => Ok(Expression::EmptyExpression),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::token::TokenType;

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
}
