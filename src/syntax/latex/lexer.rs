use super::ast::{Token, TokenKind};
use crate::syntax::text::CharStream;

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

    fn single_char(&mut self, kind: TokenKind) -> Token {
        self.stream.start_span();
        self.stream.next();
        let span = self.stream.end_span();
        Token::new(span, kind)
    }

    fn math(&mut self) -> Token {
        self.stream.start_span();
        self.stream.next();
        if self.stream.satifies(|c| *c == '$') {
            self.stream.next();
        }
        let span = self.stream.end_span();
        Token::new(span, TokenKind::Math)
    }

    fn command(&mut self) -> Token {
        let span = self.stream.command();
        Token::new(span, TokenKind::Command)
    }

    fn word(&mut self) -> Token {
        self.stream.start_span();
        self.stream.next();
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
                Some('%') => {
                    self.stream.skip_rest_of_line();
                }
                Some('{') => {
                    return Some(self.single_char(TokenKind::BeginGroup));
                }
                Some('}') => {
                    return Some(self.single_char(TokenKind::EndGroup));
                }
                Some('[') => {
                    return Some(self.single_char(TokenKind::BeginOptions));
                }
                Some(']') => {
                    return Some(self.single_char(TokenKind::EndOptions));
                }
                Some('$') => {
                    return Some(self.math());
                }
                Some(',') => {
                    return Some(self.single_char(TokenKind::Comma));
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
        && c != ','
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        protocol::{Position, Range},
        syntax::text::Span,
    };

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
        let mut lexer = Lexer::new("\\foo\\bar@baz\n\\foo*");
        verify(&mut lexer, 0, 0, "\\foo", TokenKind::Command);
        verify(&mut lexer, 0, 4, "\\bar@baz", TokenKind::Command);
        verify(&mut lexer, 1, 0, "\\foo*", TokenKind::Command);
        assert_eq!(None, lexer.next());
    }

    #[test]
    fn escape_sequence() {
        let mut lexer = Lexer::new("\\%\\**");
        verify(&mut lexer, 0, 0, "\\%", TokenKind::Command);
        verify(&mut lexer, 0, 2, "\\*", TokenKind::Command);
        verify(&mut lexer, 0, 4, "*", TokenKind::Word);
        assert_eq!(None, lexer.next());
    }

    #[test]
    fn group_delimiter() {
        let mut lexer = Lexer::new("{}[]");
        verify(&mut lexer, 0, 0, "{", TokenKind::BeginGroup);
        verify(&mut lexer, 0, 1, "}", TokenKind::EndGroup);
        verify(&mut lexer, 0, 2, "[", TokenKind::BeginOptions);
        verify(&mut lexer, 0, 3, "]", TokenKind::EndOptions);
        assert_eq!(None, lexer.next());
    }

    #[test]
    fn math() {
        let mut lexer = Lexer::new("$$ $ $");
        verify(&mut lexer, 0, 0, "$$", TokenKind::Math);
        verify(&mut lexer, 0, 3, "$", TokenKind::Math);
        verify(&mut lexer, 0, 5, "$", TokenKind::Math);
        assert_eq!(None, lexer.next());
    }

    #[test]
    fn line_comment() {
        let mut lexer = Lexer::new(" %foo \nfoo");
        verify(&mut lexer, 1, 0, "foo", TokenKind::Word);
        assert_eq!(None, lexer.next());
    }
}
