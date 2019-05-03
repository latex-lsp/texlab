mod analysis;
mod ast;
mod lexer;
mod parser;

use crate::syntax::latex::lexer::LatexLexer;
use crate::syntax::latex::parser::LatexParser;

pub use crate::syntax::latex::analysis::citation::*;
pub use crate::syntax::latex::analysis::command::*;
pub use crate::syntax::latex::analysis::environment::*;
pub use crate::syntax::latex::analysis::equation::*;
pub use crate::syntax::latex::analysis::finder::*;
pub use crate::syntax::latex::analysis::include::*;
pub use crate::syntax::latex::analysis::label::*;
pub use crate::syntax::latex::analysis::section::*;
pub use crate::syntax::latex::ast::*;
pub use crate::syntax::text::{Span, SyntaxNode};

#[derive(Debug, PartialEq, Eq, Clone)]
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
