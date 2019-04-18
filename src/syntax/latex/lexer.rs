use crate::syntax::latex::ast::{LatexToken, LatexTokenKind};
use crate::syntax::text::CharStream;

pub struct LatexLexer<'a> {
    stream: CharStream<'a>,
}

impl<'a> LatexLexer<'a> {
    pub fn new(text: &'a str) -> Self {
        LatexLexer {
            stream: CharStream::new(text),
        }
    }

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::syntax::text::Span;
    use lsp_types::{Position, Range};

    fn verify<'a>(
        lexer: &mut LatexLexer<'a>,
        line: u64,
        character: u64,
        text: &str,
        kind: LatexTokenKind,
    ) {
        let start = Position::new(line, character);
        let end = Position::new(line, character + text.chars().count() as u64);
        let range = Range::new(start, end);
        let span = Span::new(range, text.to_owned());
        let token = LatexToken::new(span, kind);
        assert_eq!(Some(token), lexer.next());
    }

    #[test]
    fn test_word() {
        let mut lexer = LatexLexer::new("foo bar baz");
        verify(&mut lexer, 0, 0, "foo", LatexTokenKind::Word);
        verify(&mut lexer, 0, 4, "bar", LatexTokenKind::Word);
        verify(&mut lexer, 0, 8, "baz", LatexTokenKind::Word);
        assert_eq!(None, lexer.next());
    }

    #[test]
    fn test_command() {
        let mut lexer = LatexLexer::new("\\foo\\bar@baz\n\\foo*");
        verify(&mut lexer, 0, 0, "\\foo", LatexTokenKind::Command);
        verify(&mut lexer, 0, 4, "\\bar@baz", LatexTokenKind::Command);
        verify(&mut lexer, 1, 0, "\\foo*", LatexTokenKind::Command);
        assert_eq!(None, lexer.next());
    }

    #[test]
    fn test_escape_sequence() {
        let mut lexer = LatexLexer::new("\\%\\**");
        verify(&mut lexer, 0, 0, "\\%", LatexTokenKind::Command);
        verify(&mut lexer, 0, 2, "\\*", LatexTokenKind::Command);
        verify(&mut lexer, 0, 4, "*", LatexTokenKind::Word);
        assert_eq!(None, lexer.next());
    }

    #[test]
    fn test_group_delimiter() {
        let mut lexer = LatexLexer::new("{}[]");
        verify(&mut lexer, 0, 0, "{", LatexTokenKind::BeginGroup);
        verify(&mut lexer, 0, 1, "}", LatexTokenKind::EndGroup);
        verify(&mut lexer, 0, 2, "[", LatexTokenKind::BeginOptions);
        verify(&mut lexer, 0, 3, "]", LatexTokenKind::EndOptions);
        assert_eq!(None, lexer.next());
    }

    #[test]
    fn test_math() {
        let mut lexer = LatexLexer::new("$$ $ $");
        verify(&mut lexer, 0, 0, "$$", LatexTokenKind::Math);
        verify(&mut lexer, 0, 3, "$", LatexTokenKind::Math);
        verify(&mut lexer, 0, 5, "$", LatexTokenKind::Math);
        assert_eq!(None, lexer.next());
    }

    #[test]
    fn test_line_comment() {
        let mut lexer = LatexLexer::new(" %foo \nfoo");
        verify(&mut lexer, 1, 0, "foo", LatexTokenKind::Word);
        assert_eq!(None, lexer.next());
    }
}
