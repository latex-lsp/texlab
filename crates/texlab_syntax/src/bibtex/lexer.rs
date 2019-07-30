use super::ast::{BibtexToken, BibtexTokenKind};
use crate::text::CharStream;

pub struct BibtexLexer<'a> {
    stream: CharStream<'a>,
}

impl<'a> BibtexLexer<'a> {
    pub fn new(text: &'a str) -> Self {
        BibtexLexer {
            stream: CharStream::new(text),
        }
    }

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
        let kind = match span.text.to_lowercase().as_ref() {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::text::Span;
    use lsp_types::{Position, Range};

    fn verify<'a>(
        lexer: &mut BibtexLexer<'a>,
        line: u64,
        character: u64,
        text: &str,
        kind: BibtexTokenKind,
    ) {
        let start = Position::new(line, character);
        let end = Position::new(line, character + text.chars().count() as u64);
        let range = Range::new(start, end);
        let span = Span::new(range, text.to_owned());
        let token = BibtexToken::new(span, kind);
        assert_eq!(Some(token), lexer.next());
    }

    #[test]
    fn test_word() {
        let mut lexer = BibtexLexer::new("foo bar baz");
        verify(&mut lexer, 0, 0, "foo", BibtexTokenKind::Word);
        verify(&mut lexer, 0, 4, "bar", BibtexTokenKind::Word);
        verify(&mut lexer, 0, 8, "baz", BibtexTokenKind::Word);
        assert_eq!(None, lexer.next());
    }

    #[test]
    fn test_command() {
        let mut lexer = BibtexLexer::new("\\foo\\bar@baz");
        verify(&mut lexer, 0, 0, "\\foo", BibtexTokenKind::Command);
        verify(&mut lexer, 0, 4, "\\bar@baz", BibtexTokenKind::Command);
        assert_eq!(None, lexer.next());
    }

    #[test]
    fn test_escape_sequence() {
        let mut lexer = BibtexLexer::new("\\foo*\n\\%\\**");
        verify(&mut lexer, 0, 0, "\\foo*", BibtexTokenKind::Command);
        verify(&mut lexer, 1, 0, "\\%", BibtexTokenKind::Command);
        verify(&mut lexer, 1, 2, "\\*", BibtexTokenKind::Command);
        verify(&mut lexer, 1, 4, "*", BibtexTokenKind::Word);
        assert_eq!(None, lexer.next());
    }

    #[test]
    fn test_delimiter() {
        let mut lexer = BibtexLexer::new("{}()\"");
        verify(&mut lexer, 0, 0, "{", BibtexTokenKind::BeginBrace);
        verify(&mut lexer, 0, 1, "}", BibtexTokenKind::EndBrace);
        verify(&mut lexer, 0, 2, "(", BibtexTokenKind::BeginParen);
        verify(&mut lexer, 0, 3, ")", BibtexTokenKind::EndParen);
        verify(&mut lexer, 0, 4, "\"", BibtexTokenKind::Quote);
        assert_eq!(None, lexer.next());
    }

    #[test]
    fn test_kind() {
        let mut lexer = BibtexLexer::new("@pReAmBlE\n@article\n@string");
        verify(&mut lexer, 0, 0, "@pReAmBlE", BibtexTokenKind::PreambleKind);
        verify(&mut lexer, 1, 0, "@article", BibtexTokenKind::EntryKind);
        verify(&mut lexer, 2, 0, "@string", BibtexTokenKind::StringKind);
        assert_eq!(None, lexer.next());
    }

    #[test]
    fn test_operator() {
        let mut lexer = BibtexLexer::new("=,#");
        verify(&mut lexer, 0, 0, "=", BibtexTokenKind::Assign);
        verify(&mut lexer, 0, 1, ",", BibtexTokenKind::Comma);
        verify(&mut lexer, 0, 2, "#", BibtexTokenKind::Concat);
        assert_eq!(None, lexer.next());
    }
}
