mod analysis;
mod cst;
mod kind;
mod lexer;
mod parser;

pub use self::{
    analysis::*,
    cst::*,
    kind::SyntaxKind::{self, *},
    parser::{parse, Parse},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LatexLanguage {}

impl rowan::Language for LatexLanguage {
    type Kind = SyntaxKind;

    fn kind_from_raw(raw: rowan::SyntaxKind) -> Self::Kind {
        assert!(raw.0 <= ROOT as u16);
        unsafe { std::mem::transmute::<u16, SyntaxKind>(raw.0) }
    }

    fn kind_to_raw(kind: Self::Kind) -> rowan::SyntaxKind {
        kind.into()
    }
}

pub type SyntaxNode = rowan::SyntaxNode<LatexLanguage>;

pub type SyntaxToken = rowan::SyntaxToken<LatexLanguage>;

pub type SyntaxElement = rowan::SyntaxElement<LatexLanguage>;
