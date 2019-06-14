use crate::syntax::latex::ast::LatexRoot;

mod analysis;
mod ast;
mod lexer;
mod parser;

pub use self::analysis::citation::*;
pub use self::analysis::command::*;
pub use self::analysis::command_definition::*;
pub use self::analysis::environment::*;
pub use self::analysis::equation::*;
pub use self::analysis::finder::*;
pub use self::analysis::include::*;
pub use self::analysis::inline::*;
pub use self::analysis::label::*;
pub use self::analysis::math_operator::*;
pub use self::analysis::section::*;
pub use self::ast::*;
use self::lexer::LatexLexer;
use self::parser::LatexParser;
use crate::syntax::text::SyntaxNode;
use lsp_types::{Position, Uri};
use std::sync::Arc;

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct LatexSyntaxTree {
    pub root: Arc<LatexRoot>,
    pub commands: Vec<Arc<LatexCommand>>,
    pub includes: Vec<LatexInclude>,
    pub components: Vec<String>,
    pub environments: Vec<LatexEnvironment>,
    pub is_standalone: bool,
    pub labels: Vec<LatexLabel>,
    pub sections: Vec<LatexSection>,
    pub citations: Vec<LatexCitation>,
    pub equations: Vec<LatexEquation>,
    pub inlines: Vec<LatexInline>,
    pub math_operators: Vec<LatexMathOperator>,
    pub command_definitions: Vec<LatexCommandDefinition>,
}

impl LatexSyntaxTree {
    pub fn new(uri: &Uri, text: &str) -> Self {
        let lexer = LatexLexer::new(text);
        let mut parser = LatexParser::new(lexer);
        let root = Arc::new(parser.root());
        let commands = LatexCommandAnalyzer::find(Arc::clone(&root));
        let includes = LatexInclude::parse_all(uri, &commands);
        let components = includes.iter().flat_map(LatexInclude::name).collect();
        let environments = LatexEnvironment::parse_all(&commands);
        let is_standalone = environments
            .iter()
            .any(|env| env.left.name().map(LatexToken::text) == Some("document"));

        let labels = LatexLabel::parse_all(&commands);
        let sections = LatexSection::parse_all(&commands);
        let citations = LatexCitation::parse_all(&commands);
        let equations = LatexEquation::parse_all(&commands);
        let inlines = LatexInlineAnalyzer::parse_all(Arc::clone(&root));
        let math_operators = LatexMathOperator::parse_all(&commands);
        let command_definitions = LatexCommandDefinition::parse_all(&commands);

        Self {
            root,
            commands,
            includes,
            components,
            environments,
            is_standalone,
            labels,
            sections,
            citations,
            equations,
            inlines,
            math_operators,
            command_definitions,
        }
    }

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
