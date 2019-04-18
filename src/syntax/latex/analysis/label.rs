use crate::syntax::latex::ast::*;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum LatexLabelKind {
    Definition,
    Reference,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexLabel<'a> {
    pub command: &'a LatexCommand,
    pub name: &'a LatexToken,
    pub kind: LatexLabelKind,
}

impl<'a> LatexLabel<'a> {
    pub fn new(command: &'a LatexCommand, name: &'a LatexToken, kind: LatexLabelKind) -> Self {
        LatexLabel {
            command,
            name,
            kind,
        }
    }
}

pub struct LatexLabelAnalyzer<'a> {
    pub labels: Vec<LatexLabel<'a>>,
}

impl<'a> LatexLabelAnalyzer<'a> {
    pub fn new() -> Self {
        LatexLabelAnalyzer { labels: Vec::new() }
    }
}

impl<'a> LatexVisitor<'a> for LatexLabelAnalyzer<'a> {
    fn visit_root(&mut self, root: &'a LatexRoot) {
        LatexWalker::walk_root(self, root);
    }

    fn visit_group(&mut self, group: &'a LatexGroup) {
        LatexWalker::walk_group(self, group);
    }

    fn visit_command(&mut self, command: &'a LatexCommand) {
        if let Some(name) = command.extract_word(0) {
            if LABEL_DEFINITION_COMMANDS.contains(&command.name.text()) {
                self.labels
                    .push(LatexLabel::new(command, name, LatexLabelKind::Definition));
            } else if LABEL_REFERENCE_COMMANDS.contains(&command.name.text()) {
                self.labels
                    .push(LatexLabel::new(command, name, LatexLabelKind::Reference));
            }
        }
        LatexWalker::walk_command(self, command);
    }

    fn visit_text(&mut self, text: &'a LatexText) {}
}

pub const LABEL_DEFINITION_COMMANDS: &'static [&'static str] = &["\\label"];

pub const LABEL_REFERENCE_COMMANDS: &'static [&'static str] = &["\\ref", "\\autoref", "\\eqref"];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::syntax::latex::LatexSyntaxTree;

    fn verify(text: &str, expected: Vec<&str>) {
        let tree = LatexSyntaxTree::from(text);
        let mut analyzer = LatexLabelAnalyzer::new();
        analyzer.visit_root(&tree.root);
        let actual: Vec<&str> = analyzer
            .labels
            .iter()
            .map(|label| label.name.text())
            .collect();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_valid() {
        verify("\\label{foo}", vec!["foo"]);
        verify("\\ref{bar}", vec!["bar"]);
    }

    #[test]
    fn test_invalid() {
        verify("\\label", vec![]);
        verify("\\label{}", vec![]);
    }

    #[test]
    fn test_unrelated() {
        verify("\\foo", vec![]);
        verify("\\foo{bar}", vec![]);
    }
}
