mod commands;
pub(super) mod types;

use logos::Logos;
use syntax::latex::SyntaxKind;

use crate::SyntaxConfig;

use self::types::{CommandName, Token};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Lexer<'a> {
    tokens: Vec<(Token, &'a str)>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str, config: &SyntaxConfig) -> Self {
        let mut tokens = tokenize(input, config);
        tokens.reverse();
        Self { tokens }
    }

    pub fn peek(&self) -> Option<Token> {
        self.tokens.last().map(|(kind, _)| *kind)
    }

    pub fn eat(&mut self) -> Option<(SyntaxKind, &'a str)> {
        let (kind, text) = self.tokens.pop()?;
        let kind = match kind {
            Token::LineBreak => SyntaxKind::LINE_BREAK,
            Token::Whitespace => SyntaxKind::WHITESPACE,
            Token::LineComment => SyntaxKind::COMMENT,
            Token::LCurly => SyntaxKind::L_CURLY,
            Token::RCurly => SyntaxKind::R_CURLY,
            Token::LBrack => SyntaxKind::L_BRACK,
            Token::RBrack => SyntaxKind::R_BRACK,
            Token::LParen => SyntaxKind::L_PAREN,
            Token::RParen => SyntaxKind::R_PAREN,
            Token::Comma => SyntaxKind::COMMA,
            Token::Eq => SyntaxKind::EQUALITY_SIGN,
            Token::Word => SyntaxKind::WORD,
            Token::Dollar => SyntaxKind::DOLLAR,
            Token::CommandName(_) => SyntaxKind::COMMAND_NAME,
        };

        Some((kind, text))
    }
}

fn tokenize<'a>(input: &'a str, config: &SyntaxConfig) -> Vec<(Token, &'a str)> {
    let mut lexer = Token::lexer(input);
    std::iter::from_fn(move || {
        let kind = lexer.next()?.unwrap();
        let text = lexer.slice();
        Some((kind, text))
    })
    .map(|(kind, text)| {
        if kind == Token::CommandName(CommandName::Generic) {
            let name = commands::classify(&text[1..], config);
            (Token::CommandName(name), text)
        } else {
            (kind, text)
        }
    })
    .collect()
}
