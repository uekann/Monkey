use crate::ast::Program;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::token::TokenType;
use std::io::{self, Write};

const PROMPT: &str = ">> ";
// pub fn start() {
//     loop {
//         let mut input = String::new();
//         print!("{}", PROMPT);
//         io::stdout().flush().unwrap();
//         io::stdin().read_line(&mut input).unwrap();
//         let mut l = Lexer::new(&input);

//         // 最初のトークンがEOFなら終了
//         let tok = l.next_token();
//         if tok.token_type == TokenType::EOF {
//             break;
//         }
//         println!("{:?}", tok);

//         loop {
//             let tok = l.next_token();
//             if tok.token_type == TokenType::EOF {
//                 break;
//             }
//             println!("{:?}", tok);
//         }
//     }
// }

pub fn start() {
    loop {
        let mut input = String::new();
        print!("{}", PROMPT);
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut input).unwrap();
        if input.trim() == "exit" {
            break;
        }
        let l = Lexer::new(&input);
        let mut p = Parser::new(l);
        let program = p.parse_program();
        match program {
            Ok(p) => {
                println!("{}", p);
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }
}
