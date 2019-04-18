pub mod analysis;
pub mod ast;
pub mod lexer;
pub mod parser;

use crate::syntax::latex::ast::LatexRoot;
use crate::syntax::latex::lexer::LatexLexer;
use crate::syntax::latex::parser::LatexParser;

pub struct LatexSyntaxTree {
    pub root: LatexRoot,
}

impl From<LatexRoot> for LatexSyntaxTree {
    fn from(root: LatexRoot) -> Self {
        LatexSyntaxTree { root }
    }
}

impl From<&str> for LatexSyntaxTree {
    fn from(text: &str) -> Self {
        let lexer = LatexLexer::new(text);
        let mut parser = LatexParser::new(lexer);
        let root = parser.root();
        LatexSyntaxTree::from(root)
    }
}
