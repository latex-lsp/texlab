mod analysis;
mod ast;
mod lexer;
mod parser;

pub use self::{analysis::*, ast::*};

use self::{lexer::Lexer, parser::Parser};
use crate::{
    protocol::{Options, Uri},
    tex::Resolver,
};
use std::path::Path;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct OpenParams<'a> {
    pub text: &'a str,
    pub uri: &'a Uri,
    pub resolver: &'a Resolver,
    pub options: &'a Options,
    pub cwd: &'a Path,
}

pub fn open(params: OpenParams) -> SymbolTable {
    let OpenParams {
        text,
        uri,
        resolver,
        options,
        cwd,
    } = params;

    let lexer = Lexer::new(text);
    let parser = Parser::new(lexer);
    let tree = parser.parse();

    let params = SymbolTableParams {
        tree,
        uri,
        resolver,
        options,
        cwd,
    };
    SymbolTable::analyze(params)
}
