use crate::token::{Token, TokenType};

#[derive(Default, Debug, Clone, Copy)]
pub struct Lexer<'a> {
    input: &'a str,
    position: usize,
    read_position: usize,
    symbol: Option<char>, // Changed type to Option<char>
}

/// Returns true if the character can be used as an identifier
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

    /// Reads the next character and updates the symbol
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

    /// Reads the next identifier
    fn read_identifier(&mut self) -> String {
        let position = self.position;
        while let Some(symbol) = self.peak_symbol() {
            if can_use_as_ident(symbol) {
                self.read_symbol();
            } else {
                break;
            }
        }
        self.input[position..self.read_position].to_string()
    }

    /// Reads the next number
    fn read_number(&mut self) -> String {
        let position = self.position;
        while let Some(symbol) = self.peak_symbol() {
            if symbol.is_numeric() {
                self.read_symbol();
            } else {
                break;
            }
        }
        self.input[position..self.read_position].to_string()
    }

    /// Skips the whitespace
    fn skip_whitespace(&mut self) -> () {
        while let Some(symbol) = self.symbol {
            if symbol.is_whitespace() {
                self.read_symbol();
            } else {
                break;
            }
        }
    }

    /// Returns the next character without updating the symbol
    fn peak_symbol(&self) -> Option<char> {
        self.input[self.read_position..]
            .char_indices()
            .next()
            .map(|(_, c)| c)
    }

    /// Returns the next token
    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();
        let token: Token = match self.symbol {
            Some('=') => {
                if self.peak_symbol() == Some('=') {
                    self.read_symbol();
                    Token {
                        token_type: TokenType::EQ,
                        literal: "==".to_string(),
                    }
                } else {
                    Token {
                        token_type: TokenType::ASSIGN,
                        literal: "=".to_string(),
                    }
                }
            }
            Some('+') => Token {
                token_type: TokenType::PLUS,
                literal: "+".to_string(),
            },
            Some('-') => Token {
                token_type: TokenType::MINUS,
                literal: "-".to_string(),
            },
            Some('!') => {
                if self.peak_symbol() == Some('=') {
                    self.read_symbol();
                    Token {
                        token_type: TokenType::NOT_EQ,
                        literal: "!=".to_string(),
                    }
                } else {
                    Token {
                        token_type: TokenType::BANG,
                        literal: "!".to_string(),
                    }
                }
            }
            Some('*') => Token {
                token_type: TokenType::ASTERISK,
                literal: "*".to_string(),
            },
            Some('/') => Token {
                token_type: TokenType::SLASH,
                literal: "/".to_string(),
            },
            Some('<') => Token {
                token_type: TokenType::LT,
                literal: "<".to_string(),
            },
            Some('>') => Token {
                token_type: TokenType::GT,
                literal: ">".to_string(),
            },
            Some(',') => Token {
                token_type: TokenType::COMMA,
                literal: ",".to_string(),
            },
            Some(';') => Token {
                token_type: TokenType::SEMICOLON,
                literal: ";".to_string(),
            },
            Some('(') => Token {
                token_type: TokenType::LPAREN,
                literal: "(".to_string(),
            },
            Some(')') => Token {
                token_type: TokenType::RPAREN,
                literal: ")".to_string(),
            },
            Some('{') => Token {
                token_type: TokenType::LBRACE,
                literal: "{".to_string(),
            },
            Some('}') => Token {
                token_type: TokenType::RBRACE,
                literal: "}".to_string(),
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
                    token_type: TokenType::lookup_ident(&literal),
                    literal: literal,
                }
            }
            None => {
                return Token {
                    token_type: TokenType::EOF,
                    literal: "".to_string(),
                }
            }
            _ => Token {
                token_type: TokenType::ILLEGAL,
                literal: self.input[self.position..self.read_position].to_string(), // Updated symbol handling
            },
        };
        self.read_symbol();
        token
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        let token: Token = self.next_token();
        if token.token_type == TokenType::EOF {
            None
        } else {
            Some(token)
        }
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
                literal: "let".to_string(),
            },
            Token {
                token_type: TokenType::IDENT,
                literal: "five".to_string(),
            },
            Token {
                token_type: TokenType::ASSIGN,
                literal: "=".to_string(),
            },
            Token {
                token_type: TokenType::INT,
                literal: "5".to_string(),
            },
            Token {
                token_type: TokenType::SEMICOLON,
                literal: ";".to_string(),
            },
            Token {
                token_type: TokenType::LET,
                literal: "let".to_string(),
            },
            Token {
                token_type: TokenType::IDENT,
                literal: "ten".to_string(),
            },
            Token {
                token_type: TokenType::ASSIGN,
                literal: "=".to_string(),
            },
            Token {
                token_type: TokenType::INT,
                literal: "10".to_string(),
            },
            Token {
                token_type: TokenType::SEMICOLON,
                literal: ";".to_string(),
            },
            Token {
                token_type: TokenType::LET,
                literal: "let".to_string(),
            },
            Token {
                token_type: TokenType::IDENT,
                literal: "add".to_string(),
            },
            Token {
                token_type: TokenType::ASSIGN,
                literal: "=".to_string(),
            },
            Token {
                token_type: TokenType::FUNCTION,
                literal: "fn".to_string(),
            },
            Token {
                token_type: TokenType::LPAREN,
                literal: "(".to_string(),
            },
            Token {
                token_type: TokenType::IDENT,
                literal: "x".to_string(),
            },
            Token {
                token_type: TokenType::COMMA,
                literal: ",".to_string(),
            },
            Token {
                token_type: TokenType::IDENT,
                literal: "y".to_string(),
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
                token_type: TokenType::IDENT,
                literal: "x".to_string(),
            },
            Token {
                token_type: TokenType::PLUS,
                literal: "+".to_string(),
            },
            Token {
                token_type: TokenType::IDENT,
                literal: "y".to_string(),
            },
            Token {
                token_type: TokenType::SEMICOLON,
                literal: ";".to_string(),
            },
            Token {
                token_type: TokenType::RBRACE,
                literal: "}".to_string(),
            },
            Token {
                token_type: TokenType::SEMICOLON,
                literal: ";".to_string(),
            },
            Token {
                token_type: TokenType::LET,
                literal: "let".to_string(),
            },
            Token {
                token_type: TokenType::IDENT,
                literal: "result".to_string(),
            },
            Token {
                token_type: TokenType::ASSIGN,
                literal: "=".to_string(),
            },
            Token {
                token_type: TokenType::IDENT,
                literal: "add".to_string(),
            },
            Token {
                token_type: TokenType::LPAREN,
                literal: "(".to_string(),
            },
            Token {
                token_type: TokenType::IDENT,
                literal: "five".to_string(),
            },
            Token {
                token_type: TokenType::COMMA,
                literal: ",".to_string(),
            },
            Token {
                token_type: TokenType::IDENT,
                literal: "ten".to_string(),
            },
            Token {
                token_type: TokenType::RPAREN,
                literal: ")".to_string(),
            },
            Token {
                token_type: TokenType::SEMICOLON,
                literal: ";".to_string(),
            },
            Token {
                token_type: TokenType::BANG,
                literal: "!".to_string(),
            },
            Token {
                token_type: TokenType::MINUS,
                literal: "-".to_string(),
            },
            Token {
                token_type: TokenType::SLASH,
                literal: "/".to_string(),
            },
            Token {
                token_type: TokenType::ASTERISK,
                literal: "*".to_string(),
            },
            Token {
                token_type: TokenType::INT,
                literal: "5".to_string(),
            },
            Token {
                token_type: TokenType::SEMICOLON,
                literal: ";".to_string(),
            },
            Token {
                token_type: TokenType::INT,
                literal: "5".to_string(),
            },
            Token {
                token_type: TokenType::LT,
                literal: "<".to_string(),
            },
            Token {
                token_type: TokenType::INT,
                literal: "10".to_string(),
            },
            Token {
                token_type: TokenType::GT,
                literal: ">".to_string(),
            },
            Token {
                token_type: TokenType::INT,
                literal: "5".to_string(),
            },
            Token {
                token_type: TokenType::SEMICOLON,
                literal: ";".to_string(),
            },
            Token {
                token_type: TokenType::IF,
                literal: "if".to_string(),
            },
            Token {
                token_type: TokenType::LPAREN,
                literal: "(".to_string(),
            },
            Token {
                token_type: TokenType::INT,
                literal: "5".to_string(),
            },
            Token {
                token_type: TokenType::LT,
                literal: "<".to_string(),
            },
            Token {
                token_type: TokenType::INT,
                literal: "10".to_string(),
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
                token_type: TokenType::RETURN,
                literal: "return".to_string(),
            },
            Token {
                token_type: TokenType::TRUE,
                literal: "true".to_string(),
            },
            Token {
                token_type: TokenType::SEMICOLON,
                literal: ";".to_string(),
            },
            Token {
                token_type: TokenType::RBRACE,
                literal: "}".to_string(),
            },
            Token {
                token_type: TokenType::ELSE,
                literal: "else".to_string(),
            },
            Token {
                token_type: TokenType::LBRACE,
                literal: "{".to_string(),
            },
            Token {
                token_type: TokenType::RETURN,
                literal: "return".to_string(),
            },
            Token {
                token_type: TokenType::FALSE,
                literal: "false".to_string(),
            },
            Token {
                token_type: TokenType::SEMICOLON,
                literal: ";".to_string(),
            },
            Token {
                token_type: TokenType::RBRACE,
                literal: "}".to_string(),
            },
            Token {
                token_type: TokenType::INT,
                literal: "10".to_string(),
            },
            Token {
                token_type: TokenType::EQ,
                literal: "==".to_string(),
            },
            Token {
                token_type: TokenType::INT,
                literal: "10".to_string(),
            },
            Token {
                token_type: TokenType::SEMICOLON,
                literal: ";".to_string(),
            },
            Token {
                token_type: TokenType::INT,
                literal: "10".to_string(),
            },
            Token {
                token_type: TokenType::NOT_EQ,
                literal: "!=".to_string(),
            },
            Token {
                token_type: TokenType::INT,
                literal: "9".to_string(),
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
                literal: "漢字".to_string(),
            },
            Token {
                token_type: TokenType::IDENT,
                literal: "😄".to_string(),
            },
            Token {
                token_type: TokenType::IDENT,
                literal: "＋".to_string(),
            },
            Token {
                token_type: TokenType::IDENT,
                literal: "🇯🇵".to_string(),
            },
            Token {
                token_type: TokenType::ILLEGAL,
                literal: "\u{001B}".to_string(),
            },
        ];
        let mut l = Lexer::new(input);
        for test in tests.iter() {
            let token: Token = l.next_token();
            assert_eq!(token, *test)
        }
    }

    #[test]
    fn test_iterator() {
        let input = "let x = 5; let y = 10; let foobar = 838383;";
        let tests = vec![
            Token {
                token_type: TokenType::LET,
                literal: "let".to_string(),
            },
            Token {
                token_type: TokenType::IDENT,
                literal: "x".to_string(),
            },
            Token {
                token_type: TokenType::ASSIGN,
                literal: "=".to_string(),
            },
            Token {
                token_type: TokenType::INT,
                literal: "5".to_string(),
            },
            Token {
                token_type: TokenType::SEMICOLON,
                literal: ";".to_string(),
            },
            Token {
                token_type: TokenType::LET,
                literal: "let".to_string(),
            },
            Token {
                token_type: TokenType::IDENT,
                literal: "y".to_string(),
            },
            Token {
                token_type: TokenType::ASSIGN,
                literal: "=".to_string(),
            },
            Token {
                token_type: TokenType::INT,
                literal: "10".to_string(),
            },
            Token {
                token_type: TokenType::SEMICOLON,
                literal: ";".to_string(),
            },
            Token {
                token_type: TokenType::LET,
                literal: "let".to_string(),
            },
            Token {
                token_type: TokenType::IDENT,
                literal: "foobar".to_string(),
            },
            Token {
                token_type: TokenType::ASSIGN,
                literal: "=".to_string(),
            },
            Token {
                token_type: TokenType::INT,
                literal: "838383".to_string(),
            },
            Token {
                token_type: TokenType::SEMICOLON,
                literal: ";".to_string(),
            },
        ];
        let l = Lexer::new(input);
        for (i, token) in l.enumerate() {
            assert_eq!(token, tests[i]);
        }
    }
}
