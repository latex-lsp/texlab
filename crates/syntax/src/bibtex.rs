mod cst;
mod kind;

pub use self::{
    cst::*,
    kind::SyntaxKind::{self, *},
};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
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
