use crate::syntax::latex::ast::*;
use std::sync::Arc;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum LatexLabelKind {
    Definition,
    Reference,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexLabel {
    pub command: Arc<LatexCommand>,
}

impl LatexLabel {
    pub fn new(command: Arc<LatexCommand>) -> Self {
        LatexLabel { command }
    }

    pub fn name(&self) -> &LatexToken {
        self.command.extract_word(0).unwrap()
    }

    pub fn kind(&self) -> LatexLabelKind {
        if LABEL_DEFINITION_COMMANDS.contains(&self.command.name.text()) {
            LatexLabelKind::Definition
        } else {
            LatexLabelKind::Reference
        }
    }

    pub fn parse(commands: &[Arc<LatexCommand>]) -> Vec<Self> {
        let mut labels = Vec::new();
        for command in commands {
            if command.has_word(0)
                && (LABEL_DEFINITION_COMMANDS.contains(&command.name.text())
                    || LABEL_REFERENCE_COMMANDS.contains(&command.name.text()))
            {
                labels.push(LatexLabel::new(Arc::clone(command)));
            }
        }
        labels
    }
}

pub static LABEL_DEFINITION_COMMANDS: &[&str] = &["\\label"];

pub static LABEL_REFERENCE_COMMANDS: &[&str] = &["\\ref", "\\autoref", "\\eqref"];

#[cfg(test)]
mod tests {
    use crate::syntax::latex::LatexSyntaxTree;

    fn verify(text: &str, expected: Vec<&str>) {
        let tree = LatexSyntaxTree::from(text);
        let actual: Vec<&str> = tree
            .labels
            .iter()
            .map(|label| label.name().text())
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
