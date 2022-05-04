use itertools::Itertools;
use logos::Logos;

use crate::syntax::token_ptr::TokenPtr;

use super::SyntaxKind::{self, *};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, PartialOrd, Ord, Logos)]
pub enum Type {
    #[regex(r"@[Pp][Rr][Ee][Aa][Mm][Bb][Ll][Ee]")]
    Preamble,

    #[regex(r"@[Ss][Tt][Rr][Ii][Nn][Gg]")]
    String,

    #[regex(r"@[Cc][Oo][Mm][Mm][Ee][Nn][Tt]")]
    Comment,

    #[error]
    Entry,
}

impl<'a> From<&'a str> for Type {
    fn from(input: &'a str) -> Self {
        Type::lexer(input).next().unwrap_or(Self::Entry)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, PartialOrd, Ord, Logos)]
pub enum Token {
    #[regex(r"\s+")]
    Whitespace,

    #[regex(r#"@[^\s\{\}\(\),#"=\\]*"#, |lex| Type::from(lex.slice()))]
    Type(Type),

    #[regex(r#"[^\s\{\}\(\),#"=\\]+"#)]
    #[error]
    Word,

    #[regex(r"\d+", priority = 2)]
    Integer,

    #[token("{")]
    LCurly,

    #[token("}")]
    RCurly,

    #[token("(")]
    LParen,

    #[token(")")]
    RParen,

    #[token(",")]
    Comma,

    #[token("#")]
    Pound,

    #[token("\"")]
    Quote,

    #[token("=")]
    Eq,

    #[regex(r"\\([^\r\n]|[@a-zA-Z:_]+\*?)?")]
    CommandName,
}

impl From<Token> for SyntaxKind {
    fn from(token: Token) -> Self {
        match token {
            Token::Whitespace => WHITESPACE,
            Token::Type(_) => TYPE,
            Token::Word => WORD,
            Token::Integer => INTEGER,
            Token::LCurly => L_CURLY,
            Token::RCurly => R_CURLY,
            Token::LParen => L_PAREN,
            Token::RParen => R_PAREN,
            Token::Comma => COMMA,
            Token::Pound => POUND,
            Token::Quote => QUOTE,
            Token::Eq => EQ,
            Token::CommandName => COMMAND_NAME,
        }
    }
}

impl From<Token> for rowan::SyntaxKind {
    fn from(token: Token) -> Self {
        SyntaxKind::from(token).into()
    }
}

pub fn tokenize<'a>(input: &'a str) -> TokenPtr<'a, Token> {
    Token::lexer(input)
        .spanned()
        .map(|(kind, range)| (kind, &input[range]))
        .collect_vec()
        .into()
}
