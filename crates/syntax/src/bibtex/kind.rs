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
