mod analysis;
mod ast;
mod lexer;
mod parser;

pub use crate::syntax::latex::analysis::citation::*;
pub use crate::syntax::latex::analysis::command::*;
pub use crate::syntax::latex::analysis::environment::*;
pub use crate::syntax::latex::analysis::equation::*;
pub use crate::syntax::latex::analysis::finder::*;
pub use crate::syntax::latex::analysis::include::*;
pub use crate::syntax::latex::analysis::label::*;
pub use crate::syntax::latex::analysis::section::*;
pub use crate::syntax::latex::ast::*;
use crate::syntax::latex::lexer::LatexLexer;
use crate::syntax::latex::parser::LatexParser;
use crate::syntax::text::SyntaxNode;
use lsp_types::Position;
use std::sync::Arc;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexSyntaxTree {
    pub root: Arc<LatexRoot>,
    pub commands: Vec<Arc<LatexCommand>>,
    pub includes: Vec<LatexInclude>,
    pub components: Vec<String>,
    pub environments: Vec<LatexEnvironment>,
    pub labels: Vec<LatexLabel>,
    pub sections: Vec<LatexSection>,
    pub citations: Vec<LatexCitation>,
    pub equations: Vec<LatexEquation>,
    pub is_standalone: bool,
}

impl LatexSyntaxTree {
    pub fn find(&self, position: Position) -> Vec<LatexNode> {
        let mut finder = LatexFinder::new(position);
        finder.visit_root(Arc::clone(&self.root));
        finder.results
    }

    pub fn find_command(&self, position: Position) -> Option<Arc<LatexCommand>> {
        for result in self.find(position) {
            if let LatexNode::Command(command) = result {
                if command.name.range().contains(position)
                    && command.name.start().character != position.character
                {
                    return Some(command);
                }
            }
        }
        None
    }
}

impl From<LatexRoot> for LatexSyntaxTree {
    fn from(root: LatexRoot) -> Self {
        let root = Arc::new(root);
        let mut analyzer = LatexCommandAnalyzer::new();
        analyzer.visit_root(Arc::clone(&root));
        let commands = analyzer.commands;
        let (includes, components) = LatexInclude::parse(&commands);
        let environments = LatexEnvironment::parse(&commands);
        let labels = LatexLabel::parse(&commands);
        let sections = LatexSection::parse(&commands);
        let citations = LatexCitation::parse(&commands);
        let equations = LatexEquation::parse(&commands);
        let is_standalone = environments
            .iter()
            .any(|env| env.left.name().map(LatexToken::text) == Some("document"));
        LatexSyntaxTree {
            root,
            commands,
            includes,
            components,
            environments,
            labels,
            sections,
            citations,
            equations,
            is_standalone,
        }
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
