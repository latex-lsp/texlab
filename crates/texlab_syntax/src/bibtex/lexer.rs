use super::ast::{Token, TokenKind};
use crate::text::CharStream;

#[derive(Debug)]
pub struct Lexer<'a> {
    stream: CharStream<'a>,
}

impl<'a> Lexer<'a> {
    pub fn new(text: &'a str) -> Self {
        Self {
            stream: CharStream::new(text),
        }
    }

    fn kind(&mut self) -> Token {
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
            "@preamble" => TokenKind::PreambleKind,
            "@string" => TokenKind::StringKind,
            _ => TokenKind::EntryKind,
        };
        Token::new(span, kind)
    }

    fn single_character(&mut self, kind: TokenKind) -> Token {
        self.stream.start_span();
        self.stream.next();
        let span = self.stream.end_span();
        Token::new(span, kind)
    }

    fn command(&mut self) -> Token {
        let span = self.stream.command();
        Token::new(span, TokenKind::Command)
    }

    fn word(&mut self) -> Token {
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
        Token::new(span, TokenKind::Word)
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Token> {
        loop {
            match self.stream.peek() {
                Some('@') => return Some(self.kind()),
                Some('=') => return Some(self.single_character(TokenKind::Assign)),
                Some(',') => return Some(self.single_character(TokenKind::Comma)),
                Some('#') => return Some(self.single_character(TokenKind::Concat)),
                Some('"') => return Some(self.single_character(TokenKind::Quote)),
                Some('{') => return Some(self.single_character(TokenKind::BeginBrace)),
                Some('}') => return Some(self.single_character(TokenKind::EndBrace)),
                Some('(') => return Some(self.single_character(TokenKind::BeginParen)),
                Some(')') => return Some(self.single_character(TokenKind::EndParen)),
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
    use texlab_protocol::{Position, Range};

    fn verify<'a>(lexer: &mut Lexer<'a>, line: u64, character: u64, text: &str, kind: TokenKind) {
        let start = Position::new(line, character);
        let end = Position::new(line, character + text.chars().count() as u64);
        let range = Range::new(start, end);
        let span = Span::new(range, text.to_owned());
        let token = Token::new(span, kind);
        assert_eq!(Some(token), lexer.next());
    }

    #[test]
    fn word() {
        let mut lexer = Lexer::new("foo bar baz");
        verify(&mut lexer, 0, 0, "foo", TokenKind::Word);
        verify(&mut lexer, 0, 4, "bar", TokenKind::Word);
        verify(&mut lexer, 0, 8, "baz", TokenKind::Word);
        assert_eq!(None, lexer.next());
    }

    #[test]
    fn command() {
        let mut lexer = Lexer::new("\\foo\\bar@baz");
        verify(&mut lexer, 0, 0, "\\foo", TokenKind::Command);
        verify(&mut lexer, 0, 4, "\\bar@baz", TokenKind::Command);
        assert_eq!(None, lexer.next());
    }

    #[test]
    fn escape_sequence() {
        let mut lexer = Lexer::new("\\foo*\n\\%\\**");
        verify(&mut lexer, 0, 0, "\\foo*", TokenKind::Command);
        verify(&mut lexer, 1, 0, "\\%", TokenKind::Command);
        verify(&mut lexer, 1, 2, "\\*", TokenKind::Command);
        verify(&mut lexer, 1, 4, "*", TokenKind::Word);
        assert_eq!(None, lexer.next());
    }

    #[test]
    fn delimiter() {
        let mut lexer = Lexer::new("{}()\"");
        verify(&mut lexer, 0, 0, "{", TokenKind::BeginBrace);
        verify(&mut lexer, 0, 1, "}", TokenKind::EndBrace);
        verify(&mut lexer, 0, 2, "(", TokenKind::BeginParen);
        verify(&mut lexer, 0, 3, ")", TokenKind::EndParen);
        verify(&mut lexer, 0, 4, "\"", TokenKind::Quote);
        assert_eq!(None, lexer.next());
    }

    #[test]
    fn kind() {
        let mut lexer = Lexer::new("@pReAmBlE\n@article\n@string");
        verify(&mut lexer, 0, 0, "@pReAmBlE", TokenKind::PreambleKind);
        verify(&mut lexer, 1, 0, "@article", TokenKind::EntryKind);
        verify(&mut lexer, 2, 0, "@string", TokenKind::StringKind);
        assert_eq!(None, lexer.next());
    }

    #[test]
    fn operator() {
        let mut lexer = Lexer::new("=,#");
        verify(&mut lexer, 0, 0, "=", TokenKind::Assign);
        verify(&mut lexer, 0, 1, ",", TokenKind::Comma);
        verify(&mut lexer, 0, 2, "#", TokenKind::Concat);
        assert_eq!(None, lexer.next());
    }
}
