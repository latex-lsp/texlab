mod ast;
mod lexer;
mod parser;
#[cfg(test)]
mod tests;

pub use self::{ast::*, parser::parse, SyntaxKind::*};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, PartialOrd, Ord)]
#[allow(non_camel_case_types)]
#[repr(u16)]
pub enum SyntaxKind {
    WHITESPACE,
    TYPE,
    WORD,
    KEY,
    INTEGER,
    L_CURLY,
    R_CURLY,
    L_PAREN,
    R_PAREN,
    COMMA,
    POUND,
    QUOTE,
    EQ,
    COMMAND_NAME,

    ERROR,
    JUNK,
    PREAMBLE,
    STRING,
    COMMENT,
    ENTRY,
    FIELD,
    LITERAL,
    CONCAT,
    CURLY_GROUP,
    QUOTE_GROUP,
    ROOT,
}

impl From<SyntaxKind> for rowan::SyntaxKind {
    fn from(kind: SyntaxKind) -> Self {
        Self(kind as u16)
    }
}

#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub enum Language {}

impl rowan::Language for Language {
    type Kind = SyntaxKind;

    fn kind_from_raw(raw: rowan::SyntaxKind) -> Self::Kind {
        assert!(raw.0 <= SyntaxKind::ROOT as u16);
        unsafe { std::mem::transmute::<u16, SyntaxKind>(raw.0) }
    }

    fn kind_to_raw(kind: Self::Kind) -> rowan::SyntaxKind {
        kind.into()
    }
}

pub type SyntaxNode = rowan::SyntaxNode<Language>;

pub type SyntaxToken = rowan::SyntaxToken<Language>;

pub type SyntaxElement = rowan::SyntaxElement<Language>;
