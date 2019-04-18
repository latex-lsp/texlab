use crate::syntax::latex::ast::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexCitation<'a> {
    pub command: &'a LatexCommand,
    pub key: &'a LatexToken,
}

impl<'a> LatexCitation<'a> {
    pub fn new(command: &'a LatexCommand, key: &'a LatexToken) -> Self {
        LatexCitation { command, key }
    }
}

pub struct LatexCitationAnalyzer<'a> {
    pub citations: Vec<LatexCitation<'a>>,
}

impl<'a> LatexCitationAnalyzer<'a> {
    pub fn new() -> Self {
        LatexCitationAnalyzer {
            citations: Vec::new(),
        }
    }
}

impl<'a> LatexVisitor<'a> for LatexCitationAnalyzer<'a> {
    fn visit_root(&mut self, root: &'a LatexRoot) {
        LatexWalker::walk_root(self, root);
    }

    fn visit_group(&mut self, group: &'a LatexGroup) {
        LatexWalker::walk_group(self, group);
    }

    fn visit_command(&mut self, command: &'a LatexCommand) {
        if CITATION_COMMANDS.contains(&command.name.text()) {
            if let Some(key) = command.extract_word(0) {
                self.citations.push(LatexCitation::new(command, key));
            }
        }
        LatexWalker::walk_command(self, command);
    }

    fn visit_text(&mut self, text: &'a LatexText) {}
}

pub const CITATION_COMMANDS: &'static [&'static str] = &[
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
    use super::*;
    use crate::syntax::latex::LatexSyntaxTree;

    fn verify(text: &str, expected: Vec<&str>) {
        let tree = LatexSyntaxTree::from(text);
        let mut analyzer = LatexCitationAnalyzer::new();
        analyzer.visit_root(&tree.root);
        let actual: Vec<&str> = analyzer
            .citations
            .iter()
            .map(|citation| citation.key.text())
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
