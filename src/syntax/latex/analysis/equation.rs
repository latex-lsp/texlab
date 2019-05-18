use crate::syntax::latex::ast::*;
use std::sync::Arc;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexEquation {
    pub left: Arc<LatexCommand>,
    pub right: Arc<LatexCommand>,
}

impl LatexEquation {
    pub fn new(left: Arc<LatexCommand>, right: Arc<LatexCommand>) -> Self {
        LatexEquation { left, right }
    }

    pub fn parse(commands: &[Arc<LatexCommand>]) -> Vec<Self> {
        let mut equations = Vec::new();
        let mut left = None;
        for command in commands {
            if command.name.text() == EQUATION_COMMANDS[0] {
                left = Some(command);
            } else if let Some(begin) = left {
                equations.push(LatexEquation::new(Arc::clone(&begin), Arc::clone(&command)));
                left = None;
            }
        }
        equations
    }
}

pub const EQUATION_COMMANDS: &'static [&'static str] = &["\\[", "\\]"];

#[cfg(test)]
mod tests {
    use crate::syntax::latex::LatexSyntaxTree;
    use crate::syntax::text::SyntaxNode;
    use lsp_types::Range;

    #[test]
    fn test_matched() {
        let tree = LatexSyntaxTree::from("\\[ foo \\]");
        let equations = tree.equations;
        assert_eq!(1, equations.len());
        assert_eq!(Range::new_simple(0, 0, 0, 2), equations[0].left.range());
        assert_eq!(Range::new_simple(0, 7, 0, 9), equations[0].right.range());
    }

    #[test]
    fn test_unmatched() {
        let tree = LatexSyntaxTree::from("\\] \\[");
        assert_eq!(tree.equations, Vec::new());
    }
}
