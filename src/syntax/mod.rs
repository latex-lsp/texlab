mod bibtex;
mod language;
mod latex;
mod text;

pub use self::bibtex::*;
pub use self::language::*;
pub use self::latex::*;
pub use self::text::*;

use crate::workspace::Uri;
use tex::Language;

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
