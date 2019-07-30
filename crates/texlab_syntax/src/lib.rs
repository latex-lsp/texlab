mod bibtex;
mod latex;
mod text;
mod language;

use lsp_types::Uri;
pub use self::text::*;
pub use self::bibtex::*;
pub use self::latex::*;
pub use self::language::*;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Language {
    Latex,
    Bibtex,
}

impl Language {
    pub fn by_extension(extension: &str) -> Option<Self> {
        match extension.to_lowercase().as_ref() {
            "tex" | "sty" | "cls" | "lco" => Some(Language::Latex),
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
    Latex(LatexSyntaxTree),
    Bibtex(BibtexSyntaxTree),
}

impl SyntaxTree {
    pub fn parse(uri: &Uri, text: &str, language: Language) -> Self {
        match language {
            Language::Latex => SyntaxTree::Latex(LatexSyntaxTree::parse(uri, text)),
            Language::Bibtex => SyntaxTree::Bibtex(text.into()),
        }
    }
}
