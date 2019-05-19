use crate::syntax::latex::ast::*;
use std::sync::Arc;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexSection {
    pub command: Arc<LatexCommand>,
    pub text: String,
    pub level: usize,
}

impl LatexSection {
    pub fn new(command: Arc<LatexCommand>, text: String, level: usize) -> Self {
        LatexSection {
            command,
            text,
            level,
        }
    }

    pub fn parse(commands: &[Arc<LatexCommand>]) -> Vec<Self> {
        let mut sections = Vec::new();
        for command in commands {
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
                    sections.push(LatexSection::new(Arc::clone(&command), text, level));
                }
            }
        }
        sections
    }
}

pub static SECTION_COMMANDS: &'static [&'static str] = &[
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
    use crate::syntax::latex::LatexSyntaxTree;

    fn verify(text: &str, expected: Vec<&str>) {
        let tree = LatexSyntaxTree::from(text);
        let actual: Vec<&str> = tree
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
