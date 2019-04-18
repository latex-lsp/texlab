use crate::syntax::latex::ast::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexEquation<'a> {
    pub left: &'a LatexCommand,
    pub right: &'a LatexCommand,
}

impl<'a> LatexEquation<'a> {
    pub fn new(left: &'a LatexCommand, right: &'a LatexCommand) -> Self {
        LatexEquation { left, right }
    }
}

pub struct LatexEquationAnalyzer<'a> {
    pub equations: Vec<LatexEquation<'a>>,
    left: Option<&'a LatexCommand>,
}

impl<'a> LatexEquationAnalyzer<'a> {
    pub fn new() -> Self {
        LatexEquationAnalyzer {
            equations: Vec::new(),
            left: None,
        }
    }
}

impl<'a> LatexVisitor<'a> for LatexEquationAnalyzer<'a> {
    fn visit_root(&mut self, root: &'a LatexRoot) {
        LatexWalker::walk_root(self, root);
    }

    fn visit_group(&mut self, group: &'a LatexGroup) {
        LatexWalker::walk_group(self, group);
    }

    fn visit_command(&mut self, command: &'a LatexCommand) {
        if command.name.text() == EQUATION_COMMANDS[0] {
            self.left = Some(command);
        } else if let Some(left) = self.left {
            self.equations.push(LatexEquation::new(left, command));
            self.left = None;
        }
        LatexWalker::walk_command(self, command);
    }

    fn visit_text(&mut self, text: &'a LatexText) {}
}

pub const EQUATION_COMMANDS: &'static [&'static str] = &["\\[", "\\]"];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::range;
    use crate::syntax::latex::LatexSyntaxTree;
    use crate::syntax::text::SyntaxNode;

    fn analyze<'a>(tree: &'a LatexSyntaxTree) -> Vec<LatexEquation<'a>> {
        let mut analyzer = LatexEquationAnalyzer::new();
        analyzer.visit_root(&tree.root);
        analyzer.equations
    }

    #[test]
    fn test_matched() {
        let tree = LatexSyntaxTree::from("\\[ foo \\]");
        let equations = analyze(&tree);
        assert_eq!(1, equations.len());
        assert_eq!(range::create(0, 0, 0, 2), equations[0].left.range());
        assert_eq!(range::create(0, 7, 0, 9), equations[0].right.range());
    }

    #[test]
    fn test_unmatched() {
        let tree = LatexSyntaxTree::from("\\] \\[");
        assert_eq!(analyze(&tree), Vec::new());
    }
}
