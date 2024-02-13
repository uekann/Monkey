use crate::token::{Token, TokenType};

#[derive(Default, Debug)]
pub struct Lexer<'a> {
    input: &'a str,
    position: usize,
    read_position: usize,
    symbol: Option<char>, // Changed type to Option<char>
}

fn can_use_as_ident(c: char) -> bool {
    !(c.is_ascii_digit() || c.is_whitespace() || c.is_control() || c.is_ascii_punctuation())
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Lexer<'a> {
        let mut l = Lexer {
            input,
            ..Default::default()
        };
        l.read_symbol();
        l
    }
    fn read_symbol(&mut self) -> () {
        self.position = self.read_position;
        match self.input[self.read_position..].char_indices().next() {
            Some((u, c)) => {
                self.read_position += u + c.len_utf8();
                self.symbol = Some(c);
            }
            None => {
                self.symbol = None;
                self.read_position += 1;
            }
        }
    }

    fn read_identifier(&mut self) -> &'a str {
        let position = self.position;
        while let Some(symbol) = self.peak_symbol() {
            if can_use_as_ident(symbol) {
                self.read_symbol();
            } else {
                break;
            }
        }
        &self.input[position..self.read_position]
    }

    fn read_number(&mut self) -> &'a str {
        let position = self.position;
        while let Some(symbol) = self.peak_symbol() {
            if symbol.is_numeric() {
                self.read_symbol();
            } else {
                break;
            }
        }
        &self.input[position..self.read_position]
    }

    fn skip_whitespace(&mut self) -> () {
        while let Some(symbol) = self.symbol {
            if symbol.is_whitespace() {
                self.read_symbol();
            } else {
                break;
            }
        }
    }

    fn peak_symbol(&self) -> Option<char> {
        self.input[self.read_position..]
            .char_indices()
            .next()
            .map(|(_, c)| c)
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();
        let token = match self.symbol {
            Some('=') => {
                if self.peak_symbol() == Some('=') {
                    self.read_symbol();
                    Token {
                        token_type: TokenType::EQ,
                        literal: "==",
                    }
                } else {
                    Token {
                        token_type: TokenType::ASSIGN,
                        literal: "=",
                    }
                }
            }
            Some('+') => Token {
                token_type: TokenType::PLUS,
                literal: "+",
            },
            Some('-') => Token {
                token_type: TokenType::MINUS,
                literal: "-",
            },
            Some('!') => {
                if self.peak_symbol() == Some('=') {
                    self.read_symbol();
                    Token {
                        token_type: TokenType::NOT_EQ,
                        literal: "!=",
                    }
                } else {
                    Token {
                        token_type: TokenType::BANG,
                        literal: "!",
                    }
                }
            }
            Some('*') => Token {
                token_type: TokenType::ASTERISK,
                literal: "*",
            },
            Some('/') => Token {
                token_type: TokenType::SLASH,
                literal: "/",
            },
            Some('<') => Token {
                token_type: TokenType::LT,
                literal: "<",
            },
            Some('>') => Token {
                token_type: TokenType::GT,
                literal: ">",
            },
            Some(',') => Token {
                token_type: TokenType::COMMA,
                literal: ",",
            },
            Some(';') => Token {
                token_type: TokenType::SEMICOLON,
                literal: ";",
            },
            Some('(') => Token {
                token_type: TokenType::LPAREN,
                literal: "(",
            },
            Some(')') => Token {
                token_type: TokenType::RPAREN,
                literal: ")",
            },
            Some('{') => Token {
                token_type: TokenType::LBRACE,
                literal: "{",
            },
            Some('}') => Token {
                token_type: TokenType::RBRACE,
                literal: "}",
            },
            Some(c) if c.is_ascii_digit() => {
                let literal = self.read_number();
                Token {
                    token_type: TokenType::INT,
                    literal: literal,
                }
            }
            Some(c) if can_use_as_ident(c) => {
                let literal = self.read_identifier();
                Token {
                    token_type: TokenType::lookup_ident(literal),
                    literal,
                }
            }
            None => {
                return Token {
                    token_type: TokenType::EOF,
                    literal: "",
                }
            }
            _ => Token {
                token_type: TokenType::ILLEGAL,
                literal: &self.input[self.position..self.read_position], // Updated symbol handling
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
        let input = "
            let five = 5;
            let ten = 10;
            
            let add = fn(x, y) {
                x + y;
            };
            let result = add(five, ten);

            !-/*5;
            5 < 10 > 5;

            if (5 < 10) {
                return true;
            } else {
                return false;
            }

            10 == 10;
            10 != 9;
            ";
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
                token_type: TokenType::BANG,
                literal: "!",
            },
            Token {
                token_type: TokenType::MINUS,
                literal: "-",
            },
            Token {
                token_type: TokenType::SLASH,
                literal: "/",
            },
            Token {
                token_type: TokenType::ASTERISK,
                literal: "*",
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
                token_type: TokenType::INT,
                literal: "5",
            },
            Token {
                token_type: TokenType::LT,
                literal: "<",
            },
            Token {
                token_type: TokenType::INT,
                literal: "10",
            },
            Token {
                token_type: TokenType::GT,
                literal: ">",
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
                token_type: TokenType::IF,
                literal: "if",
            },
            Token {
                token_type: TokenType::LPAREN,
                literal: "(",
            },
            Token {
                token_type: TokenType::INT,
                literal: "5",
            },
            Token {
                token_type: TokenType::LT,
                literal: "<",
            },
            Token {
                token_type: TokenType::INT,
                literal: "10",
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
                token_type: TokenType::RETURN,
                literal: "return",
            },
            Token {
                token_type: TokenType::TRUE,
                literal: "true",
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
                token_type: TokenType::ELSE,
                literal: "else",
            },
            Token {
                token_type: TokenType::LBRACE,
                literal: "{",
            },
            Token {
                token_type: TokenType::RETURN,
                literal: "return",
            },
            Token {
                token_type: TokenType::FALSE,
                literal: "false",
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
                token_type: TokenType::INT,
                literal: "10",
            },
            Token {
                token_type: TokenType::EQ,
                literal: "==",
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
                token_type: TokenType::INT,
                literal: "10",
            },
            Token {
                token_type: TokenType::NOT_EQ,
                literal: "!=",
            },
            Token {
                token_type: TokenType::INT,
                literal: "9",
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
    fn test_next_token3() {
        let input = "漢字 😄 ＋ 🇯🇵 \u{001B}";
        let tests = vec![
            Token {
                token_type: TokenType::IDENT,
                literal: "漢字",
            },
            Token {
                token_type: TokenType::IDENT,
                literal: "😄",
            },
            Token {
                token_type: TokenType::IDENT,
                literal: "＋",
            },
            Token {
                token_type: TokenType::IDENT,
                literal: "🇯🇵",
            },
            Token {
                token_type: TokenType::ILLEGAL,
                literal: "\u{001B}",
            },
        ];
        let mut l = Lexer::new(input);
        for test in tests.iter() {
            let token: Token = l.next_token();
            assert_eq!(token, *test)
        }
    }
}
