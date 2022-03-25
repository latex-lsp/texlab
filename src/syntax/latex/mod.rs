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
pub enum Language {}

impl cstree::Language for Language {
    type Kind = SyntaxKind;

    fn kind_from_raw(raw: cstree::SyntaxKind) -> Self::Kind {
        assert!(raw.0 <= ROOT as u16);
        unsafe { std::mem::transmute::<u16, SyntaxKind>(raw.0) }
    }

    fn kind_to_raw(kind: Self::Kind) -> cstree::SyntaxKind {
        kind.into()
    }
}

pub type SyntaxNode = cstree::ResolvedNode<Language>;

pub type SyntaxToken = cstree::ResolvedToken<Language>;

pub type SyntaxElement = cstree::ResolvedElement<Language>;

pub type SyntaxElementRef<'a> = cstree::ResolvedElementRef<'a, Language>;
