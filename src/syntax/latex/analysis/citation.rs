use crate::syntax::latex::ast::*;
use std::sync::Arc;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexCitation {
    pub command: Arc<LatexCommand>,
}

impl LatexCitation {
    pub fn new(command: Arc<LatexCommand>) -> LatexCitation {
        Self { command }
    }

    pub fn key(&self) -> &LatexToken {
        self.command.extract_word(0).unwrap()
    }

    pub fn parse_all(commands: &[Arc<LatexCommand>]) -> Vec<Self> {
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
