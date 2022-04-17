use logos::Logos;

use super::kind::SyntaxKind;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, PartialOrd, Ord, Logos)]
#[allow(non_camel_case_types)]
#[allow(clippy::upper_case_acronyms)]
#[repr(u16)]
enum LogosToken {
    #[regex(r"\s+")]
    WHITESPACE = 2,

    #[regex(r"@[Pp][Rr][Ee][Aa][Mm][Bb][Ll][Ee]")]
    PREAMBLE_TYPE,

    #[regex(r"@[Ss][Tt][Rr][Ii][Nn][Gg]")]
    STRING_TYPE,

    #[regex(r"@[Cc][Oo][Mm][Mm][Ee][Nn][Tt]")]
    COMMENT_TYPE,

    #[regex(r"@[!\$\&\*\+\-\./:;<>\?@\[\]\\\^_`\|\~a-zA-Z][!\$\&\*\+\-\./:;<>\?@\[\]\\\^_`\|\~a-zA-Z0-9]*|@")]
    ENTRY_TYPE,

    #[regex(r#"[^\s\{\}\(\),#"=\\]+"#)]
    #[error]
    WORD,

    #[token("{")]
    L_CURLY,

    #[token("}")]
    R_CURLY,

    #[token("(")]
    L_PAREN,

    #[token(")")]
    R_PAREN,

    #[token(",")]
    COMMA,

    #[token("#")]
    HASH,

    #[token("\"")]
    QUOTE,

    #[token("=")]
    EQUALITY_SIGN,

    #[regex(r"\\([^\r\n]|[@a-zA-Z:_]+\*?)?")]
    COMMAND_NAME,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Lexer<'a> {
    tokens: Vec<(SyntaxKind, &'a str)>,
}

impl<'a> Lexer<'a> {
    pub fn new(text: &'a str) -> Self {
        let mut tokens = Vec::new();
        let mut lexer = LogosToken::lexer(text);
        while let Some(kind) = lexer.next() {
            tokens.push((
                unsafe { std::mem::transmute::<LogosToken, SyntaxKind>(kind) },
                lexer.slice(),
            ));
        }
        tokens.reverse();
        Self { tokens }
    }

    pub fn peek(&self) -> Option<SyntaxKind> {
        self.tokens.last().map(|(kind, _)| *kind)
    }

    pub fn consume(&mut self) -> Option<(SyntaxKind, &'a str)> {
        self.tokens.pop()
    }
}
