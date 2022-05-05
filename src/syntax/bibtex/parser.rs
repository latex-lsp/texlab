use rowan::{GreenNode, GreenNodeBuilder};

use crate::syntax::token_ptr::TokenPtr;

use super::{
    lexer::{tokenize, Token, Type},
    SyntaxKind::{self, *},
};

struct Parser<'a> {
    ptr: TokenPtr<'a, Token>,
    ast: GreenNodeBuilder<'static>,
}

impl<'a> Parser<'a> {
    pub fn parse(mut self) -> GreenNode {
        self.ast.start_node(ROOT.into());

        while let Some(kind) = self.ptr.current() {
            match kind {
                Token::Whitespace => self.bump(),
                Token::Type(Type::Preamble) => self.preamble(),
                Token::Type(Type::String) => self.string_def(),
                Token::Type(Type::Entry) => self.entry(),
                Token::Type(Type::Comment) => self.comment(),
                Token::Word
                | Token::Integer
                | Token::LCurly
                | Token::RCurly
                | Token::LParen
                | Token::RParen
                | Token::Comma
                | Token::Pound
                | Token::Quote
                | Token::Eq
                | Token::CommandName => self.junk(),
            };
        }

        self.ast.finish_node();
        self.ast.finish()
    }

    fn bump(&mut self) {
        let (kind, text) = self.ptr.bump();
        self.ast.token(kind.into(), text);
    }

    fn bump_and_trivia(&mut self) {
        self.bump();
        self.trivia();
    }

    fn bump_and_remap(&mut self, new_kind: SyntaxKind) {
        let (_, text) = self.ptr.bump();
        self.ast.token(new_kind.into(), text);
    }

    fn trivia(&mut self) {
        while self.ptr.at(Token::Whitespace) {
            self.bump();
        }
    }

    fn junk(&mut self) {
        self.ast.start_node(JUNK.into());

        while self.ptr.at_cond(|kind| !matches!(kind, Token::Type(_))) {
            self.bump();
        }

        self.ast.finish_node();
    }

    fn comment(&mut self) {
        self.ast.start_node(COMMENT.into());
        self.bump_and_trivia();

        while self.ptr.at_cond(|kind| !matches!(kind, Token::Type(_))) {
            self.bump();
        }

        self.ast.finish_node();
    }

    fn preamble(&mut self) {
        self.ast.start_node(PREAMBLE.into());
        self.bump_and_trivia();
        self.left_delimiter();
        self.value();
        self.right_delimiter();
        self.ast.finish_node();
    }

    fn string_def(&mut self) {
        self.ast.start_node(STRING.into());
        self.bump_and_trivia();
        self.left_delimiter();
        self.key();
        self.eq();
        self.value();
        self.right_delimiter();
        self.ast.finish_node();
    }

    fn entry(&mut self) {
        self.ast.start_node(ENTRY.into());
        self.bump_and_trivia();
        self.left_delimiter();
        self.key();
        self.comma();
        while self.field().is_some() {}
        self.right_delimiter();
        self.ast.finish_node();
    }

    fn field(&mut self) -> Option<()> {
        if !self.ptr.at(Token::Word) {
            return None;
        }

        self.ast.start_node(FIELD.into());
        self.key().unwrap();
        self.eq();
        self.value();
        self.comma();
        self.ast.finish_node();
        Some(())
    }

    fn key(&mut self) -> Option<()> {
        if !self.ptr.at(Token::Word) {
            return None;
        }

        self.bump_and_remap(KEY);
        self.trivia();
        Some(())
    }

    fn left_delimiter(&mut self) -> Option<()> {
        self.token(|kind| matches!(kind, Token::LCurly | Token::LParen))
    }

    fn right_delimiter(&mut self) -> Option<()> {
        self.token(|kind| matches!(kind, Token::RCurly | Token::RParen))
    }

    fn eq(&mut self) -> Option<()> {
        self.token(|kind| kind == Token::Eq)
    }

    fn comma(&mut self) -> Option<()> {
        self.token(|kind| kind == Token::Comma)
    }

    fn token(&mut self, predicate: impl FnOnce(Token) -> bool) -> Option<()> {
        if !self.ptr.at_cond(predicate) {
            return None;
        }

        self.bump_and_trivia();
        Some(())
    }

    fn value(&mut self) -> Option<()> {
        let checkpoint = self.ast.checkpoint();
        match self.ptr.current()? {
            Token::Whitespace
            | Token::Type(_)
            | Token::LParen
            | Token::RParen
            | Token::Comma
            | Token::Pound
            | Token::Eq
            | Token::RCurly => return None,
            Token::LCurly => self.curly_group(),
            Token::Quote => self.quote_group(),
            Token::Integer | Token::Word | Token::CommandName => self.literal(),
        };

        if self.ptr.at(Token::Pound) {
            self.ast.start_node_at(checkpoint, CONCAT.into());
            self.bump_and_trivia();
            self.value();
            self.ast.finish_node();
        }

        Some(())
    }

    fn curly_group(&mut self) {
        self.ast.start_node(CURLY_GROUP.into());
        self.bump_and_trivia();

        while let Some(kind) = self.ptr.current() {
            match kind {
                Token::Whitespace
                | Token::Type(_)
                | Token::Word
                | Token::Integer
                | Token::LParen
                | Token::RParen
                | Token::Comma
                | Token::Eq
                | Token::CommandName => self.bump(),
                Token::LCurly => self.curly_group(),
                Token::Quote => self.quote_group(),
                Token::Pound => break,
                Token::RCurly => {
                    self.bump_and_trivia();
                    break;
                }
            };
        }

        self.ast.finish_node();
    }

    fn quote_group(&mut self) {
        self.ast.start_node(QUOTE_GROUP.into());
        self.bump_and_trivia();

        while let Some(kind) = self.ptr.current() {
            match kind {
                Token::Whitespace
                | Token::Type(_)
                | Token::Word
                | Token::Integer
                | Token::LParen
                | Token::RParen
                | Token::Comma
                | Token::Eq
                | Token::CommandName
                | Token::RCurly => self.bump(),
                Token::LCurly => self.curly_group(),
                Token::Pound => break,
                Token::Quote => {
                    self.bump_and_trivia();
                    break;
                }
            };
        }

        self.ast.finish_node();
    }

    fn literal(&mut self) {
        self.ast.start_node(LITERAL.into());
        self.bump_and_trivia();
        self.ast.finish_node();
    }
}

pub fn parse(input: &str) -> GreenNode {
    let ptr = tokenize(input);
    let ast = GreenNodeBuilder::new();
    let parser = Parser { ptr, ast };
    parser.parse()
}
