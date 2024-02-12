// token.rsの呼び出し

use crate::token::{Token, TokenType};

#[derive(Default, Debug)]
pub struct Lexer<'a> {
    input: &'a str,
    position: usize,
    read_position: usize,
    symbol: Option<&'a str>,
}

impl<'a> Lexer<'a> {
    fn new(input: &'a str) -> Lexer<'a> {
        let mut l = Lexer {
            input,
            ..Default::default()
        };
        l.read_symbol();
        l
    }
    fn read_symbol(&mut self) -> () {
        self.symbol = self.input.get(self.read_position..self.read_position + 1);
        self.position = self.read_position;
        self.read_position += 1;
    }
    fn next_token(&mut self) -> Token {
        let token = match self.symbol {
            Some("=") => Token {
                token_type: TokenType::ASSIGN,
                literal: "=".to_string(),
            },
            Some("+") => Token {
                token_type: TokenType::PLUS,
                literal: "+".to_string(),
            },
            Some(",") => Token {
                token_type: TokenType::COMMA,
                literal: ",".to_string(),
            },
            Some(";") => Token {
                token_type: TokenType::SEMICOLON,
                literal: ";".to_string(),
            },
            Some("(") => Token {
                token_type: TokenType::LPAREN,
                literal: "(".to_string(),
            },
            Some(")") => Token {
                token_type: TokenType::RPAREN,
                literal: ")".to_string(),
            },
            Some("{") => Token {
                token_type: TokenType::LBRACE,
                literal: "{".to_string(),
            },
            Some("}") => Token {
                token_type: TokenType::RBRACE,
                literal: "}".to_string(),
            },
            Some(_) => panic!("unexpected symbol"),
            None => Token {
                token_type: TokenType::EOF,
                literal: "".to_string(),
            },
        };
        self.read_symbol();
        token
    }
}

#[cfg(test)]
mod test {
    use crate::token::{Token, TokenType};

    use super::Lexer;

    #[test]
    fn test_next_token() {
        let input = "=+(){},;".to_string();
        let tests = vec![
            Token {
                token_type: TokenType::ASSIGN,
                literal: "=".to_string(),
            },
            Token {
                token_type: TokenType::PLUS,
                literal: "+".to_string(),
            },
            Token {
                token_type: TokenType::LPAREN,
                literal: "(".to_string(),
            },
            Token {
                token_type: TokenType::RPAREN,
                literal: ")".to_string(),
            },
            Token {
                token_type: TokenType::LBRACE,
                literal: "{".to_string(),
            },
            Token {
                token_type: TokenType::RBRACE,
                literal: "}".to_string(),
            },
            Token {
                token_type: TokenType::COMMA,
                literal: ",".to_string(),
            },
            Token {
                token_type: TokenType::SEMICOLON,
                literal: ";".to_string(),
            },
            Token {
                token_type: TokenType::EOF,
                literal: "".to_string(),
            },
        ];

        let mut l = Lexer::new(&input);
        for test in tests.iter() {
            let token: Token = l.next_token();
            assert_eq!(token, *test)
        }
    }
}
