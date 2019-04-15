use crate::syntax::latex::ast::{LatexToken, LatexTokenKind};
use crate::syntax::text::CharStream;

pub struct LatexLexer<'a> {
    stream: CharStream<'a>,
}

impl<'a> From<CharStream<'a>> for LatexLexer<'a> {
    fn from(stream: CharStream<'a>) -> Self {
        LatexLexer { stream }
    }
}

impl<'a> From<&'a str> for LatexLexer<'a> {
    fn from(text: &'a str) -> Self {
        let stream = CharStream::new(text);
        LatexLexer::from(stream)
    }
}

impl<'a> LatexLexer<'a> {
    fn single_char(&mut self, kind: LatexTokenKind) -> LatexToken {
        self.stream.start_span();
        self.stream.next();
        let span = self.stream.end_span();
        LatexToken::new(span, kind)
    }

    fn math(&mut self) -> LatexToken {
        self.stream.start_span();
        self.stream.next();
        if self.stream.satifies(|c| *c == '$') {
            self.stream.next();
        }
        let span = self.stream.end_span();
        LatexToken::new(span, LatexTokenKind::Math)
    }

    fn command(&mut self) -> LatexToken {
        let span = self.stream.command();
        LatexToken::new(span, LatexTokenKind::Command)
    }

    fn word(&mut self) -> LatexToken {
        self.stream.start_span();
        self.stream.next();
        while self.stream.satifies(|c| is_word_char(*c)) {
            self.stream.next();
        }

        let span = self.stream.end_span();
        LatexToken::new(span, LatexTokenKind::Word)
    }
}

impl<'a> Iterator for LatexLexer<'a> {
    type Item = LatexToken;

    fn next(&mut self) -> Option<LatexToken> {
        loop {
            match self.stream.peek() {
                Some('%') => {
                    self.stream.skip_rest_of_line();
                }
                Some('{') => {
                    return Some(self.single_char(LatexTokenKind::BeginGroup));
                }
                Some('}') => {
                    return Some(self.single_char(LatexTokenKind::EndGroup));
                }
                Some('[') => {
                    return Some(self.single_char(LatexTokenKind::BeginOptions));
                }
                Some(']') => {
                    return Some(self.single_char(LatexTokenKind::EndOptions));
                }
                Some('$') => {
                    return Some(self.math());
                }
                Some('\\') => {
                    return Some(self.command());
                }
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

fn is_word_char(c: char) -> bool {
    !c.is_whitespace()
        && c != '%'
        && c != '{'
        && c != '}'
        && c != '['
        && c != ']'
        && c != '\\'
        && c != '$'
}
