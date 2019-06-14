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
        Self {
            command,
            text,
            level,
        }
    }

    pub fn parse_all(commands: &[Arc<LatexCommand>]) -> Vec<Self> {
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

pub static SECTION_COMMANDS: &[&str] = &[
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
