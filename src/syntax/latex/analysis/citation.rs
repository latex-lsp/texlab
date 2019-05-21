use crate::syntax::latex::ast::*;
use std::sync::Arc;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexCitation {
    pub command: Arc<LatexCommand>,
}

impl LatexCitation {
    pub fn new(command: Arc<LatexCommand>) -> LatexCitation {
        LatexCitation { command }
    }

    pub fn key(&self) -> &LatexToken {
        self.command.extract_word(0).unwrap()
    }

    pub fn parse(commands: &[Arc<LatexCommand>]) -> Vec<Self> {
        let mut citations = Vec::new();
        for command in commands {
            if CITATION_COMMANDS.contains(&command.name.text()) && command.has_word(0) {
                citations.push(LatexCitation::new(Arc::clone(&command)));
            }
        }
        citations
    }
}

pub static CITATION_COMMANDS: &[&str] = &[
    "\\cite",
    "\\cite*",
    "\\Cite",
    "\\nocite",
    "\\citet",
    "\\citep",
    "\\citet*",
    "\\citep*",
    "\\citeauthor",
    "\\citeauthor*",
    "\\Citeauthor",
    "\\Citeauthor*",
    "\\citetitle",
    "\\citetitle*",
    "\\citeyear",
    "\\citeyear*",
    "\\citedate",
    "\\citedate*",
    "\\citeurl",
    "\\fullcite",
    "\\citeyearpar",
    "\\citealt",
    "\\citealp",
    "\\citetext",
    "\\parencite",
    "\\parencite*",
    "\\Parencite",
    "\\footcite",
    "\\footfullcite",
    "\\footcitetext",
    "\\textcite",
    "\\Textcite",
    "\\smartcite",
    "\\Smartcite",
    "\\supercite",
    "\\autocite",
    "\\Autocite",
    "\\autocite*",
    "\\Autocite*",
    "\\volcite",
    "\\Volcite",
    "\\pvolcite",
    "\\Pvolcite",
    "\\fvolcite",
    "\\ftvolcite",
    "\\svolcite",
    "\\Svolcite",
    "\\tvolcite",
    "\\Tvolcite",
    "\\avolcite",
    "\\Avolcite",
    "\\notecite",
    "\\notecite",
    "\\pnotecite",
    "\\Pnotecite",
    "\\fnotecite",
];

#[cfg(test)]
mod tests {
    use crate::syntax::latex::LatexSyntaxTree;

    fn verify(text: &str, expected: Vec<&str>) {
        let tree = LatexSyntaxTree::from(text);
        let actual: Vec<&str> = tree
            .citations
            .iter()
            .map(|citation| citation.key().text())
            .collect();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_valid() {
        verify("\\cite{foo}", vec!["foo"]);
        verify("\\Cite{bar}", vec!["bar"]);
    }

    #[test]
    fn test_invalid() {
        verify("\\cite", vec![]);
        verify("\\cite{}", vec![]);
    }

    #[test]
    fn test_unrelated() {
        verify("\\foo", vec![]);
        verify("\\foo{bar}", vec![]);
    }
}
