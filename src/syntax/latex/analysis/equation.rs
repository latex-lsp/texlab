use crate::syntax::latex::ast::*;
use crate::syntax::text::SyntaxNode;
use lsp_types::Range;
use std::sync::Arc;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexEquation {
    pub left: Arc<LatexCommand>,
    pub right: Arc<LatexCommand>,
}

impl SyntaxNode for LatexEquation {
    fn range(&self) -> Range {
        Range::new(self.left.start(), self.right.end())
    }
}

impl LatexEquation {
    pub fn new(left: Arc<LatexCommand>, right: Arc<LatexCommand>) -> Self {
        Self { left, right }
    }

    pub fn parse_all(commands: &[Arc<LatexCommand>]) -> Vec<Self> {
        let mut equations = Vec::new();
        let mut left = None;
        for command in commands {
            let name = command.name.text();
            if name == "\\[" || name == "\\(" {
                left = Some(command);
            } else if name == "\\]" || name == "\\)" {
                if let Some(begin) = left {
                    equations.push(LatexEquation::new(Arc::clone(&begin), Arc::clone(&command)));
                    left = None;
                }
            }
        }
        equations
    }
}
