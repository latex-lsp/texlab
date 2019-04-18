use crate::syntax::latex::ast::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexSection<'a> {
    pub command: &'a LatexCommand,
    pub text: String,
    pub level: usize,
}

impl<'a> LatexSection<'a> {
    pub fn new(command: &'a LatexCommand, text: String, level: usize) -> Self {
        LatexSection {
            command,
            text,
            level,
        }
    }
}

pub struct LatexSectionAnalyzer<'a> {
    pub sections: Vec<LatexSection<'a>>,
}

impl<'a> LatexSectionAnalyzer<'a> {
    pub fn new() -> Self {
        LatexSectionAnalyzer {
            sections: Vec::new(),
        }
    }
}

impl<'a> LatexVisitor<'a> for LatexSectionAnalyzer<'a> {
    fn visit_root(&mut self, root: &'a LatexRoot) {
        LatexWalker::walk_root(self, root);
    }

    fn visit_group(&mut self, group: &'a LatexGroup) {
        LatexWalker::walk_group(self, group);
    }

    fn visit_command(&mut self, command: &'a LatexCommand) {
        if SECTION_COMMANDS.contains(&command.name.text()) {
            if let Some(text) = command.extract_content(0) {
                let mut level = 0;
                for name in SECTION_COMMANDS {
                    if &command.name.text() == name {
                        break;
                    }
                    level += 1;
                }
                level /= 2;
                self.sections.push(LatexSection::new(command, text, level));
            }
        }
        LatexWalker::walk_command(self, command);
    }

    fn visit_text(&mut self, text: &'a LatexText) {}
}

pub const SECTION_COMMANDS: &'static [&'static str] = &[
    "\\chapter",
    "\\chapter*",
    "\\section",
    "\\section*",
    "\\subsection",
    "\\subsection*",
    "\\subsubsection",
    "\\subsubsection*",
    "\\paragraph",
    "\\paragraph*",
    "\\subparagraph",
    "\\subparagraph*",
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::syntax::latex::LatexSyntaxTree;

    fn verify(text: &str, expected: Vec<&str>) {
        let tree = LatexSyntaxTree::from(text);
        let mut analyzer = LatexSectionAnalyzer::new();
        analyzer.visit_root(&tree.root);
        let actual: Vec<&str> = analyzer
            .sections
            .iter()
            .map(|section| section.text.as_ref())
            .collect();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_valid() {
        verify("\\section{foo bar}", vec!["foo bar"]);
        verify("\\chapter{bar}", vec!["bar"]);
    }

    #[test]
    fn test_invalid() {
        verify("\\section", vec![]);
        verify("\\section{}", vec![]);
    }

    #[test]
    fn test_unrelated() {
        verify("\\foo", vec![]);
        verify("\\foo{bar}", vec![]);
    }
}
