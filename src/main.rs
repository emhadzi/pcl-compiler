mod lexer;
mod parser;

use std::io::Read;
use lexer::{Lexer, TokenKind};

fn main() {
    let mut src = String::new();
    std::io::stdin().read_to_string(&mut src).unwrap();
    let mut lexer = Lexer::new(&src);

    // Filter out whitespaces
    let _tokens = lexer
        .lex()
        .into_iter()
        .filter(|t| t.0 != TokenKind::Whitespace)
        .collect::<Vec<_>>();

    // Call parser
}
