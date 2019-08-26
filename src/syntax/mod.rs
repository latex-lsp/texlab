mod bibtex;
mod language;
mod latex;
mod text;

pub use self::bibtex::*;
pub use self::language::*;
pub use self::latex::*;
pub use self::text::*;

use crate::workspace::Uri;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Language {
    Latex,
    Bibtex,
}

impl Language {
    pub fn by_extension(extension: &str) -> Option<Self> {
        match extension.to_lowercase().as_ref() {
            "tex" | "sty" | "cls" | "lco" | "aux" => Some(Language::Latex),
            "bib" => Some(Language::Bibtex),
            _ => None,
        }
    }

    pub fn by_language_id(language_id: &str) -> Option<Self> {
        match language_id {
            "latex" | "tex" => Some(Language::Latex),
            "bibtex" | "bib" => Some(Language::Bibtex),
            _ => None,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum SyntaxTree {
    Latex(Box<LatexSyntaxTree>),
    Bibtex(Box<BibtexSyntaxTree>),
}

impl SyntaxTree {
    pub fn parse(uri: &Uri, text: &str, language: Language) -> Self {
        match language {
            Language::Latex => SyntaxTree::Latex(Box::new(LatexSyntaxTree::parse(uri, text))),
            Language::Bibtex => SyntaxTree::Bibtex(Box::new(text.into())),
        }
    }
}
