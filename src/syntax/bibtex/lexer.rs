use crate::syntax::bibtex::ast::{BibtexToken, BibtexTokenKind};
use crate::syntax::text::CharStream;

pub struct BibtexLexer<'a> {
    stream: CharStream<'a>,
}

impl<'a> From<CharStream<'a>> for BibtexLexer<'a> {
    fn from(stream: CharStream<'a>) -> Self {
        BibtexLexer { stream }
    }
}

impl<'a> From<&'a str> for BibtexLexer<'a> {
    fn from(text: &'a str) -> Self {
        let stream = CharStream::new(text);
        BibtexLexer::from(stream)
    }
}

impl<'a> BibtexLexer<'a> {
    fn kind(&mut self) -> BibtexToken {
        fn is_type_char(c: char) -> bool {
            c >= 'a' && c <= 'z' || c >= 'A' && c <= 'Z'
        }

        self.stream.start_span();
        self.stream.next().unwrap();
        while self.stream.satifies(|c| is_type_char(*c)) {
            self.stream.next();
        }
        let span = self.stream.end_span();
        let kind = match span.text.as_ref() {
            "@preamble" => BibtexTokenKind::PreambleKind,
            "@string" => BibtexTokenKind::StringKind,
            _ => BibtexTokenKind::EntryKind,
        };
        BibtexToken::new(span, kind)
    }

    fn single_character(&mut self, kind: BibtexTokenKind) -> BibtexToken {
        self.stream.start_span();
        self.stream.next();
        let span = self.stream.end_span();
        BibtexToken::new(span, kind)
    }

    fn command(&mut self) -> BibtexToken {
        let span = self.stream.command();
        BibtexToken::new(span, BibtexTokenKind::Command)
    }

    fn word(&mut self) -> BibtexToken {
        fn is_word_char(c: char) -> bool {
            !c.is_whitespace()
                && c != '@'
                && c != '='
                && c != ','
                && c != '#'
                && c != '"'
                && c != '{'
                && c != '}'
                && c != '('
                && c != ')'
        }

        self.stream.start_span();
        while self.stream.satifies(|c| is_word_char(*c)) {
            self.stream.next();
        }
        let span = self.stream.end_span();
        BibtexToken::new(span, BibtexTokenKind::Word)
    }
}

impl<'a> Iterator for BibtexLexer<'a> {
    type Item = BibtexToken;

    fn next(&mut self) -> Option<BibtexToken> {
        loop {
            match self.stream.peek() {
                Some('@') => return Some(self.kind()),
                Some('=') => return Some(self.single_character(BibtexTokenKind::Assign)),
                Some(',') => return Some(self.single_character(BibtexTokenKind::Comma)),
                Some('#') => return Some(self.single_character(BibtexTokenKind::Concat)),
                Some('"') => return Some(self.single_character(BibtexTokenKind::Quote)),
                Some('{') => return Some(self.single_character(BibtexTokenKind::BeginBrace)),
                Some('}') => return Some(self.single_character(BibtexTokenKind::EndBrace)),
                Some('(') => return Some(self.single_character(BibtexTokenKind::BeginParen)),
                Some(')') => return Some(self.single_character(BibtexTokenKind::EndParen)),
                Some('\\') => return Some(self.command()),
                Some(c) => {
                    if c.is_whitespace() {
                        self.stream.next();
                    } else {
                        return Some(self.word());
                    }
                }
                None => {
                    return None;
                }
            }
        }
    }
}
