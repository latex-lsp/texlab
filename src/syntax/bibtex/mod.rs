mod ast;
mod lexer;
mod parser;

pub use self::ast::*;

use self::{lexer::Lexer, parser::Parser};

pub fn open(text: &str) -> Tree {
    let lexer = Lexer::new(text);
    let parser = Parser::new(lexer);
    parser.parse()
}
