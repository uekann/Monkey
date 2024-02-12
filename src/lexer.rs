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

    fn read_identifier(&mut self) -> &'a str {
        let position = self.position;
        while self.symbol.is_some() && self.symbol.unwrap().chars().nth(0).unwrap().is_alphabetic()
        {
            self.read_symbol();
        }
        self.input.get(position..self.position).unwrap()
    }

    fn skip_whitespace(&mut self) -> () {
        while self.symbol.is_some() && self.symbol.unwrap().chars().nth(0).unwrap().is_whitespace()
        {
            self.read_symbol();
        }
    }

    fn next_token(&mut self) -> Token {
        self.skip_whitespace();
        let token = match self.symbol {
            Some("=") => Token {
                token_type: TokenType::ASSIGN,
                literal: "=",
            },
            Some("+") => Token {
                token_type: TokenType::PLUS,
                literal: "+",
            },
            Some(",") => Token {
                token_type: TokenType::COMMA,
                literal: ",",
            },
            Some(";") => Token {
                token_type: TokenType::SEMICOLON,
                literal: ";",
            },
            Some("(") => Token {
                token_type: TokenType::LPAREN,
                literal: "(",
            },
            Some(")") => Token {
                token_type: TokenType::RPAREN,
                literal: ")",
            },
            Some("{") => Token {
                token_type: TokenType::LBRACE,
                literal: "{",
            },
            Some("}") => Token {
                token_type: TokenType::RBRACE,
                literal: "}",
            },
            Some(c) if c.chars().nth(0).unwrap().is_alphabetic() => {
                let literal = self.read_identifier();
                match literal {
                    "fn" => {
                        return Token {
                            token_type: TokenType::FUNCTION,
                            literal,
                        }
                    }
                    "let" => {
                        return Token {
                            token_type: TokenType::LET,
                            literal,
                        }
                    }
                    _ => {
                        return Token {
                            token_type: TokenType::IDENT,
                            literal,
                        }
                    }
                }
            }
            Some(c) if c.chars().nth(0).unwrap().is_numeric() => {
                let position = self.position;
                while self.symbol.is_some()
                    && self.symbol.unwrap().chars().nth(0).unwrap().is_numeric()
                {
                    self.read_symbol();
                }
                return Token {
                    token_type: TokenType::INT,
                    literal: self.input.get(position..self.position).unwrap(),
                };
            }
            None => Token {
                token_type: TokenType::EOF,
                literal: "",
            },
            _ => Token {
                token_type: TokenType::ILLEGAL,
                literal: self.symbol.unwrap_or(""),
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
    fn test_next_token1() {
        let input = "=+(){},;";
        let tests = vec![
            Token {
                token_type: TokenType::ASSIGN,
                literal: "=",
            },
            Token {
                token_type: TokenType::PLUS,
                literal: "+",
            },
            Token {
                token_type: TokenType::LPAREN,
                literal: "(",
            },
            Token {
                token_type: TokenType::RPAREN,
                literal: ")",
            },
            Token {
                token_type: TokenType::LBRACE,
                literal: "{",
            },
            Token {
                token_type: TokenType::RBRACE,
                literal: "}",
            },
            Token {
                token_type: TokenType::COMMA,
                literal: ",",
            },
            Token {
                token_type: TokenType::SEMICOLON,
                literal: ";",
            },
            Token {
                token_type: TokenType::EOF,
                literal: "",
            },
        ];

        let mut l = Lexer::new(input);
        for test in tests.iter() {
            let token: Token = l.next_token();
            assert_eq!(token, *test)
        }
    }

    #[test]
    fn test_next_token2() {
        let input = "let five = 5;\nlet ten = 10;\n\nlet add = fn(x, y) {\n  x + y;\n};\nlet result = add(five, ten);\n";
        let tests = vec![
            Token {
                token_type: TokenType::LET,
                literal: "let",
            },
            Token {
                token_type: TokenType::IDENT,
                literal: "five",
            },
            Token {
                token_type: TokenType::ASSIGN,
                literal: "=",
            },
            Token {
                token_type: TokenType::INT,
                literal: "5",
            },
            Token {
                token_type: TokenType::SEMICOLON,
                literal: ";",
            },
            Token {
                token_type: TokenType::LET,
                literal: "let",
            },
            Token {
                token_type: TokenType::IDENT,
                literal: "ten",
            },
            Token {
                token_type: TokenType::ASSIGN,
                literal: "=",
            },
            Token {
                token_type: TokenType::INT,
                literal: "10",
            },
            Token {
                token_type: TokenType::SEMICOLON,
                literal: ";",
            },
            Token {
                token_type: TokenType::LET,
                literal: "let",
            },
            Token {
                token_type: TokenType::IDENT,
                literal: "add",
            },
            Token {
                token_type: TokenType::ASSIGN,
                literal: "=",
            },
            Token {
                token_type: TokenType::FUNCTION,
                literal: "fn",
            },
            Token {
                token_type: TokenType::LPAREN,
                literal: "(",
            },
            Token {
                token_type: TokenType::IDENT,
                literal: "x",
            },
            Token {
                token_type: TokenType::COMMA,
                literal: ",",
            },
            Token {
                token_type: TokenType::IDENT,
                literal: "y",
            },
            Token {
                token_type: TokenType::RPAREN,
                literal: ")",
            },
            Token {
                token_type: TokenType::LBRACE,
                literal: "{",
            },
            Token {
                token_type: TokenType::IDENT,
                literal: "x",
            },
            Token {
                token_type: TokenType::PLUS,
                literal: "+",
            },
            Token {
                token_type: TokenType::IDENT,
                literal: "y",
            },
            Token {
                token_type: TokenType::SEMICOLON,
                literal: ";",
            },
            Token {
                token_type: TokenType::RBRACE,
                literal: "}",
            },
            Token {
                token_type: TokenType::SEMICOLON,
                literal: ";",
            },
            Token {
                token_type: TokenType::LET,
                literal: "let",
            },
            Token {
                token_type: TokenType::IDENT,
                literal: "result",
            },
            Token {
                token_type: TokenType::ASSIGN,
                literal: "=",
            },
            Token {
                token_type: TokenType::IDENT,
                literal: "add",
            },
            Token {
                token_type: TokenType::LPAREN,
                literal: "(",
            },
            Token {
                token_type: TokenType::IDENT,
                literal: "five",
            },
            Token {
                token_type: TokenType::COMMA,
                literal: ",",
            },
            Token {
                token_type: TokenType::IDENT,
                literal: "ten",
            },
            Token {
                token_type: TokenType::RPAREN,
                literal: ")",
            },
            Token {
                token_type: TokenType::SEMICOLON,
                literal: ";",
            },
            Token {
                token_type: TokenType::EOF,
                literal: "",
            },
        ];

        let mut l = Lexer::new(input);
        for test in tests.iter() {
            let token: Token = l.next_token();
            assert_eq!(token, *test)
        }
    }
}
