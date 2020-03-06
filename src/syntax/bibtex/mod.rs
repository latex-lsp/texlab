mod ast;
mod formatter;
mod lexer;
mod parser;

pub use self::{ast::*, formatter::*};

use self::{lexer::Lexer, parser::Parser};

pub fn open(text: &str) -> Tree {
    let lexer = Lexer::new(text);
    let parser = Parser::new(lexer);
    parser.parse()
}
