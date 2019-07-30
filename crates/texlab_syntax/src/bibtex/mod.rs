mod ast;
mod finder;
mod lexer;
mod parser;

use crate::bibtex::lexer::BibtexLexer;
use crate::bibtex::parser::BibtexParser;

pub use crate::bibtex::ast::*;
pub use crate::bibtex::finder::*;
use lsp_types::Position;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BibtexSyntaxTree {
    pub root: BibtexRoot,
}

impl BibtexSyntaxTree {
    pub fn entries(&self) -> Vec<&BibtexEntry> {
        let mut entries: Vec<&BibtexEntry> = Vec::new();
        for declaration in &self.root.children {
            if let BibtexDeclaration::Entry(entry) = declaration {
                entries.push(&entry);
            }
        }
        entries
    }

    pub fn find(&self, position: Position) -> Vec<BibtexNode> {
        let mut finder = BibtexFinder::new(position);
        finder.visit_root(&self.root);
        finder.results
    }
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
        parser.root().into()
    }
}
