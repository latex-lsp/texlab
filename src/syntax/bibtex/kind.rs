#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, PartialOrd, Ord)]
#[allow(non_camel_case_types)]
#[repr(u16)]
pub enum SyntaxKind {
    ERROR = 0,
    MISSING,

    WHITESPACE,
    PREAMBLE_TYPE,
    STRING_TYPE,
    COMMENT_TYPE,
    ENTRY_TYPE,
    WORD,
    L_CURLY,
    R_CURLY,
    L_PAREN,
    R_PAREN,
    COMMA,
    HASH,
    QUOTE,
    EQUALITY_SIGN,
    COMMAND_NAME,

    JUNK,
    PREAMBLE,
    STRING,
    COMMENT,
    ENTRY,
    KEY,
    FIELD,
    VALUE,
    TOKEN,
    BRACE_GROUP,
    QUOTE_GROUP,
    ROOT,
}

impl SyntaxKind {
    pub fn is_type(&self) -> bool {
        use SyntaxKind::*;
        matches!(
            self,
            PREAMBLE_TYPE | STRING_TYPE | COMMENT_TYPE | ENTRY_TYPE
        )
    }
}

impl From<SyntaxKind> for rowan::SyntaxKind {
    fn from(kind: SyntaxKind) -> Self {
        Self(kind as u16)
    }
}
