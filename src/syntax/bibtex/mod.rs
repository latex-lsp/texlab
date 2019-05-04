mod ast;
mod finder;
mod lexer;
mod parser;

use crate::syntax::bibtex::lexer::BibtexLexer;
use crate::syntax::bibtex::parser::BibtexParser;

pub use crate::syntax::bibtex::ast::*;
pub use crate::syntax::bibtex::finder::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BibtexSyntaxTree {
    pub root: BibtexRoot,
}

impl From<BibtexRoot> for BibtexSyntaxTree {
    fn from(root: BibtexRoot) -> Self {
        BibtexSyntaxTree { root }
    }
}

impl From<&str> for BibtexSyntaxTree {
    fn from(text: &str) -> Self {
        let lexer = BibtexLexer::new(text);
        let mut parser = BibtexParser::new(lexer);
        let root = parser.root();
        BibtexSyntaxTree::from(root)
    }
}
