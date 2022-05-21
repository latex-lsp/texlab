use logos::Logos;

use super::SyntaxKind::{self, *};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, Logos)]
pub enum RootToken {
    #[token(r"@preamble", ignore(ascii_case))]
    Preamble,

    #[token(r"@string", ignore(ascii_case))]
    String,

    #[token(r"@comment", ignore(ascii_case))]
    Comment,

    #[regex(r"@[a-zA-Z]*")]
    Entry,

    #[regex(r"[^@]+")]
    #[error]
    Junk,
}

impl From<RootToken> for SyntaxKind {
    fn from(token: RootToken) -> Self {
        match token {
            RootToken::Preamble | RootToken::String | RootToken::Comment | RootToken::Entry => TYPE,
            RootToken::Junk => JUNK,
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, Logos)]
pub enum BodyToken {
    #[regex(r"\s+")]
    Whitespace,

    #[token("{")]
    #[token("(")]
    LDelim,

    #[token("}")]
    #[token(")")]
    RDelim,

    #[token(",")]
    Comma,

    #[token("=")]
    Eq,

    #[regex(r"[^\s\(\)\{\}@,=]+")]
    Name,

    #[error]
    Error,
}

impl From<BodyToken> for SyntaxKind {
    fn from(token: BodyToken) -> Self {
        match token {
            BodyToken::Whitespace => WHITESPACE,
            BodyToken::LDelim => L_DELIM,
            BodyToken::RDelim => R_DELIM,
            BodyToken::Comma => COMMA,
            BodyToken::Eq => EQ,
            BodyToken::Name => NAME,
            BodyToken::Error => unreachable!(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, Logos)]
pub enum ValueToken {
    #[regex(r"\s+")]
    Whitespace,

    #[token("#")]
    Pound,

    #[token(",")]
    Comma,

    #[token("{")]
    LCurly,

    #[token("}")]
    RCurly,

    #[token("\"")]
    Quote,

    #[regex(r"\d+", priority = 2)]
    Integer,

    #[regex(r#"[^\s"\{\},]+"#)]
    #[error]
    Name,
}

impl From<ValueToken> for SyntaxKind {
    fn from(token: ValueToken) -> Self {
        match token {
            ValueToken::Whitespace => WHITESPACE,
            ValueToken::Pound => POUND,
            ValueToken::Comma => COMMA,
            ValueToken::LCurly => L_CURLY,
            ValueToken::RCurly => R_CURLY,
            ValueToken::Quote => QUOTE,
            ValueToken::Integer => INTEGER,
            ValueToken::Name => NAME,
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, Logos)]
pub enum ContentToken {
    #[regex(r"\s+")]
    Whitespace,

    #[token(",")]
    Comma,

    #[token("{")]
    LCurly,

    #[token("}")]
    RCurly,

    #[token("\"")]
    Quote,

    #[regex(r"\d+", priority = 2)]
    Integer,

    #[token(r#"~"#)]
    Nbsp,

    #[token(r#"\`"#)]
    #[token(r#"\'"#)]
    #[token(r#"\^"#)]
    #[token(r#"\""#)]
    #[token(r#"\H"#)]
    #[token(r#"\~"#)]
    #[token(r#"\c"#)]
    #[token(r#"\k"#)]
    #[token(r#"\="#)]
    #[token(r#"\b"#)]
    #[token(r#"\."#)]
    #[token(r#"\d"#)]
    #[token(r#"\r"#)]
    #[token(r#"\u"#)]
    #[token(r#"\v"#)]
    #[token(r#"\t"#)]
    AccentName,

    #[regex(r"\\([^\r\n]|[@a-zA-Z:_]+\*?)?")]
    CommandName,

    #[regex(r#"[^\s"\{\}\\~,]+"#)]
    #[error]
    Word,
}

impl From<ContentToken> for SyntaxKind {
    fn from(token: ContentToken) -> Self {
        match token {
            ContentToken::Whitespace => WHITESPACE,
            ContentToken::Comma => COMMA,
            ContentToken::LCurly => L_CURLY,
            ContentToken::RCurly => R_CURLY,
            ContentToken::Quote => QUOTE,
            ContentToken::Integer => INTEGER,
            ContentToken::Nbsp => NBSP,
            ContentToken::AccentName => ACCENT_NAME,
            ContentToken::CommandName => COMMAND_NAME,
            ContentToken::Word => WORD,
        }
    }
}
