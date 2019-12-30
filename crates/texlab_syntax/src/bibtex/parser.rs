use super::ast::*;
use std::iter::Peekable;

pub struct BibtexParser<I: Iterator<Item = BibtexToken>> {
    tokens: Peekable<I>,
}

impl<I: Iterator<Item = BibtexToken>> BibtexParser<I> {
    pub fn new(tokens: I) -> Self {
        Self {
            tokens: tokens.peekable(),
        }
    }

    pub fn root(&mut self) -> BibtexRoot {
        let mut children = Vec::new();
        while let Some(ref token) = self.tokens.peek() {
            match token.kind {
                BibtexTokenKind::PreambleKind => {
                    let preamble = Box::new(self.preamble());
                    children.push(BibtexDeclaration::Preamble(preamble));
                }
                BibtexTokenKind::StringKind => {
                    let string = Box::new(self.string());
                    children.push(BibtexDeclaration::String(string));
                }
                BibtexTokenKind::EntryKind => {
                    let entry = Box::new(self.entry());
                    children.push(BibtexDeclaration::Entry(entry));
                }
                _ => {
                    let comment = BibtexComment::new(self.tokens.next().unwrap());
                    children.push(BibtexDeclaration::Comment(Box::new(comment)));
                }
            }
        }
        BibtexRoot::new(children)
    }

    fn preamble(&mut self) -> BibtexPreamble {
        let ty = self.tokens.next().unwrap();

        let left = self.expect2(BibtexTokenKind::BeginBrace, BibtexTokenKind::BeginParen);
        if left.is_none() {
            return BibtexPreamble::new(ty, None, None, None);
        }

        if !self.can_match_content() {
            return BibtexPreamble::new(ty, left, None, None);
        }
        let content = self.content();

        let right = self.expect2(BibtexTokenKind::EndBrace, BibtexTokenKind::EndParen);
        BibtexPreamble::new(ty, left, Some(content), right)
    }

    fn string(&mut self) -> BibtexString {
        let ty = self.tokens.next().unwrap();

        let left = self.expect2(BibtexTokenKind::BeginBrace, BibtexTokenKind::BeginParen);
        if left.is_none() {
            return BibtexString::new(ty, None, None, None, None, None);
        }

        let name = self.expect1(BibtexTokenKind::Word);
        if name.is_none() {
            return BibtexString::new(ty, left, None, None, None, None);
        }

        let assign = self.expect1(BibtexTokenKind::Assign);
        if assign.is_none() {
            return BibtexString::new(ty, left, name, None, None, None);
        }

        if !self.can_match_content() {
            return BibtexString::new(ty, left, name, assign, None, None);
        }
        let value = self.content();

        let right = self.expect2(BibtexTokenKind::EndBrace, BibtexTokenKind::EndParen);
        BibtexString::new(ty, left, name, assign, Some(value), right)
    }

    fn entry(&mut self) -> BibtexEntry {
        let ty = self.tokens.next().unwrap();

        let left = self.expect2(BibtexTokenKind::BeginBrace, BibtexTokenKind::BeginParen);
        if left.is_none() {
            return BibtexEntry::new(ty, None, None, None, Vec::new(), None);
        }

        let name = self.expect1(BibtexTokenKind::Word);
        if name.is_none() {
            return BibtexEntry::new(ty, left, None, None, Vec::new(), None);
        }

        let comma = self.expect1(BibtexTokenKind::Comma);
        if comma.is_none() {
            return BibtexEntry::new(ty, left, name, None, Vec::new(), None);
        }

        let mut fields = Vec::new();
        while self.next_of_kind(BibtexTokenKind::Word) {
            fields.push(self.field());
        }

        let right = self.expect2(BibtexTokenKind::EndBrace, BibtexTokenKind::EndParen);
        BibtexEntry::new(ty, left, name, comma, fields, right)
    }

    fn field(&mut self) -> BibtexField {
        let name = self.tokens.next().unwrap();

        let assign = self.expect1(BibtexTokenKind::Assign);
        if assign.is_none() {
            return BibtexField::new(name, None, None, None);
        }

        if !self.can_match_content() {
            return BibtexField::new(name, assign, None, None);
        }
        let content = self.content();

        let comma = self.expect1(BibtexTokenKind::Comma);
        BibtexField::new(name, assign, Some(content), comma)
    }

    fn content(&mut self) -> BibtexContent {
        let token = self.tokens.next().unwrap();
        let left = match token.kind {
            BibtexTokenKind::PreambleKind
            | BibtexTokenKind::StringKind
            | BibtexTokenKind::EntryKind
            | BibtexTokenKind::Word
            | BibtexTokenKind::Assign
            | BibtexTokenKind::Comma
            | BibtexTokenKind::BeginParen
            | BibtexTokenKind::EndParen => BibtexContent::Word(BibtexWord::new(token)),
            BibtexTokenKind::Command => BibtexContent::Command(BibtexCommand::new(token)),
            BibtexTokenKind::Quote => {
                let mut children = Vec::new();
                while self.can_match_content() {
                    if self.next_of_kind(BibtexTokenKind::Quote) {
                        break;
                    }
                    children.push(self.content());
                }
                let right = self.expect1(BibtexTokenKind::Quote);
                BibtexContent::QuotedContent(BibtexQuotedContent::new(token, children, right))
            }
            BibtexTokenKind::BeginBrace => {
                let mut children = Vec::new();
                while self.can_match_content() {
                    children.push(self.content());
                }
                let right = self.expect1(BibtexTokenKind::EndBrace);
                BibtexContent::BracedContent(BibtexBracedContent::new(token, children, right))
            }
            _ => unreachable!(),
        };
        if let Some(operator) = self.expect1(BibtexTokenKind::Concat) {
            let right = if self.can_match_content() {
                Some(self.content())
            } else {
                None
            };
            BibtexContent::Concat(Box::new(BibtexConcat::new(left, operator, right)))
        } else {
            left
        }
    }

    fn can_match_content(&mut self) -> bool {
        if let Some(ref token) = self.tokens.peek() {
            match token.kind {
                BibtexTokenKind::PreambleKind
                | BibtexTokenKind::StringKind
                | BibtexTokenKind::EntryKind
                | BibtexTokenKind::Word
                | BibtexTokenKind::Command
                | BibtexTokenKind::Assign
                | BibtexTokenKind::Comma
                | BibtexTokenKind::Quote
                | BibtexTokenKind::BeginBrace
                | BibtexTokenKind::BeginParen
                | BibtexTokenKind::EndParen => true,
                BibtexTokenKind::Concat | BibtexTokenKind::EndBrace => false,
            }
        } else {
            false
        }
    }

    fn expect1(&mut self, kind: BibtexTokenKind) -> Option<BibtexToken> {
        if let Some(ref token) = self.tokens.peek() {
            if token.kind == kind {
                return self.tokens.next();
            }
        }
        None
    }

    fn expect2(&mut self, kind1: BibtexTokenKind, kind2: BibtexTokenKind) -> Option<BibtexToken> {
        if let Some(ref token) = self.tokens.peek() {
            if token.kind == kind1 || token.kind == kind2 {
                return self.tokens.next();
            }
        }
        None
    }

    fn next_of_kind(&mut self, kind: BibtexTokenKind) -> bool {
        if let Some(token) = self.tokens.peek() {
            token.kind == kind
        } else {
            false
        }
    }
}
