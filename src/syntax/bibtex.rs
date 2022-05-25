mod ast;
mod lexer;
mod parser;
#[cfg(test)]
mod tests;

pub use self::{ast::*, parser::parse, SyntaxKind::*};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
#[allow(non_camel_case_types)]
#[repr(u16)]
pub enum SyntaxKind {
    WHITESPACE,
    JUNK,
    L_DELIM,
    R_DELIM,
    L_CURLY,
    R_CURLY,
    COMMA,
    POUND,
    QUOTE,
    EQ,
    TYPE,
    WORD,
    NAME,
    INTEGER,
    NBSP,
    ACCENT_NAME,
    COMMAND_NAME,

    PREAMBLE,
    STRING,
    ENTRY,
    FIELD,
    VALUE,
    LITERAL,
    JOIN,
    ACCENT,
    COMMAND,
    CURLY_GROUP,
    QUOTE_GROUP,
    ROOT,
}

impl From<SyntaxKind> for rowan::SyntaxKind {
    fn from(kind: SyntaxKind) -> Self {
        Self(kind as u16)
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub enum Lang {}

impl rowan::Language for Lang {
    type Kind = SyntaxKind;

    fn kind_from_raw(raw: rowan::SyntaxKind) -> Self::Kind {
        assert!(raw.0 <= SyntaxKind::ROOT as u16);
        unsafe { std::mem::transmute::<u16, SyntaxKind>(raw.0) }
    }

    fn kind_to_raw(kind: Self::Kind) -> rowan::SyntaxKind {
        kind.into()
    }
}

pub type SyntaxNode = rowan::SyntaxNode<Lang>;

pub type SyntaxToken = rowan::SyntaxToken<Lang>;

pub type SyntaxElement = rowan::SyntaxElement<Lang>;
