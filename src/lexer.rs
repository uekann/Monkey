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
    fn new(input: &'a str) -> Lexer<'a> {
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
        while let Some(symbol) = self.symbol {
            if can_use_as_ident(symbol) {
                self.read_symbol();
            } else {
                break;
            }
        }
        &self.input[position..self.position]
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

    fn next_token(&mut self) -> Token {
        self.skip_whitespace();
        let token = match self.symbol {
            Some('=') => Token {
                token_type: TokenType::ASSIGN,
                literal: "=",
            },
            Some('+') => Token {
                token_type: TokenType::PLUS,
                literal: "+",
            },
            Some('-') => Token {
                token_type: TokenType::MINUS,
                literal: "-",
            },
            Some('!') => Token {
                token_type: TokenType::BANG,
                literal: "!",
            },
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
                let position = self.position;
                while let Some(symbol) = self.symbol {
                    if symbol.is_numeric() {
                        self.read_symbol();
                    } else {
                        break;
                    }
                }
                return Token {
                    token_type: TokenType::INT,
                    literal: &self.input[position..self.position],
                };
            }
            Some(c) if can_use_as_ident(c) => {
                let literal = self.read_identifier();
                let token_type = TokenType::lookup_ident(literal);
                return Token {
                    token_type,
                    literal,
                };
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

    #[test]
    fn test_next_token3() {
        let input = "漢字 😄 ＋ 🇯🇵";
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
        ];
        let mut l = Lexer::new(input);
        for test in tests.iter() {
            let token: Token = l.next_token();
            assert_eq!(token, *test)
        }
    }
}
