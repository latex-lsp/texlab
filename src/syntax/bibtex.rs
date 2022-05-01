mod cst;
mod kind;
mod lexer;
mod parser;

pub use self::{
    cst::*,
    kind::SyntaxKind::{self, *},
    parser::{parse, Parse},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BibtexLanguage {}

impl rowan::Language for BibtexLanguage {
    type Kind = SyntaxKind;

    fn kind_from_raw(raw: rowan::SyntaxKind) -> Self::Kind {
        assert!(raw.0 <= ROOT as u16);
        unsafe { std::mem::transmute::<u16, SyntaxKind>(raw.0) }
    }

    fn kind_to_raw(kind: Self::Kind) -> rowan::SyntaxKind {
        kind.into()
    }
}

pub type SyntaxNode = rowan::SyntaxNode<BibtexLanguage>;

pub type SyntaxToken = rowan::SyntaxToken<BibtexLanguage>;

pub type SyntaxElement = rowan::SyntaxElement<BibtexLanguage>;
