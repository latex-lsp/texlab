use lsp_types::{Position, Range};
use std::iter::Peekable;
use std::str::CharIndices;

pub trait SyntaxNode {
    fn range(&self) -> Range;

    fn start(&self) -> Position {
        self.range().start
    }

    fn end(&self) -> Position {
        self.range().end
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Span {
    pub range: Range,
    pub text: String,
}

impl Span {
    pub fn new(range: Range, text: String) -> Self {
        Span { range, text }
    }
}

impl SyntaxNode for Span {
    fn range(&self) -> Range {
        self.range
    }
}

pub struct CharStream<'a> {
    text: &'a str,
    chars: Peekable<CharIndices<'a>>,
    pub current_position: Position,
    pub current_index: usize,
    start_position: Position,
    start_index: usize,
}

impl<'a> CharStream<'a> {
    pub fn new(text: &'a str) -> Self {
        CharStream {
            text,
            chars: text.char_indices().peekable(),
            current_position: Position::new(0, 0),
            current_index: 0,
            start_position: Position::new(0, 0),
            start_index: 0,
        }
    }

    pub fn peek(&mut self) -> Option<char> {
        self.chars.peek().map(|(_, c)| *c)
    }

    pub fn satifies<P: FnOnce(&char) -> bool>(&mut self, predicate: P) -> bool {
        self.peek().filter(predicate).is_some()
    }

    pub fn skip_rest_of_line(&mut self) {
        loop {
            match self.peek() {
                Some('\n') => {
                    self.next();
                    break;
                }
                Some(_) => {
                    self.next();
                }
                None => {
                    break;
                }
            }
        }
    }

    pub fn start_span(&mut self) {
        self.start_index = self.current_index;
        self.start_position = self.current_position;
    }

    pub fn end_span(&mut self) -> Span {
        let range = Range::new(self.start_position, self.current_position);
        let text = &self.text[self.start_index..self.current_index];
        Span::new(range, text.to_owned())
    }

    pub fn seek(&mut self, position: Position) {
        while self.current_position < position {
            self.next();
        }
    }

    pub fn command(&mut self) -> Span {
        self.start_span();
        self.next();
        let mut escape = true;
        while self.satifies(|c| is_command_char(*c)) {
            self.next();
            escape = false;
        }

        if let Some(c) = self.peek() {
            if c != '\r' && c != '\n' && (escape || c == '*') {
                self.next();
            }
        }

        self.end_span()
    }

    fn update_position(&mut self, c: char) {
        if c == '\n' {
            self.current_position.line += 1;
            self.current_position.character = 0;
        } else {
            self.current_position.character += 1;
        }
    }

    pub fn extract(text: &'a str, range: Range) -> String {
        let mut stream = Self::new(text);
        stream.seek(range.start);
        stream.start_span();
        stream.seek(range.end);
        stream.end_span().text
    }
}

impl<'a> Iterator for CharStream<'a> {
    type Item = char;

    fn next(&mut self) -> Option<char> {
        if let Some((i, c)) = self.chars.next() {
            self.current_index = i + c.len_utf8();
            self.update_position(c);
            Some(c)
        } else {
            None
        }
    }
}

fn is_command_char(c: char) -> bool {
    c >= 'a' && c <= 'z' || c >= 'A' && c <= 'Z' || c == '@'
}

pub fn titlelize(string: &str) -> String {
    let mut chars = string.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().chain(chars).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::range::RangeExt;

    #[test]
    fn test_peek() {
        let mut stream = CharStream::new("ab\nc");
        assert_eq!(Some('a'), stream.peek());
        assert_eq!(Some('a'), stream.next());
        assert_eq!(Some('b'), stream.peek());
        assert_eq!(Some('b'), stream.next());
        assert_eq!(Some('\n'), stream.peek());
        assert_eq!(Some('\n'), stream.next());
        assert_eq!(Some('c'), stream.peek());
        assert_eq!(Some('c'), stream.next());
        assert_eq!(None, stream.peek());
        assert_eq!(None, stream.next());
    }

    #[test]
    fn test_span() {
        let mut stream = CharStream::new("abc\ndef");
        stream.next();
        stream.start_span();
        stream.next();
        stream.next();
        let span = stream.end_span();
        assert_eq!(
            Span::new(Range::new_simple(0, 1, 0, 3), "bc".to_owned()),
            span
        );
        assert_eq!(Position::new(0, 1), span.start());
        assert_eq!(Position::new(0, 3), span.end());
    }

    #[test]
    fn test_span_unicode() {
        let mut stream = CharStream::new("😀😃😄😁");
        stream.next();
        stream.start_span();
        stream.next();
        stream.next();
        let span = stream.end_span();
        assert_eq!(
            Span::new(Range::new_simple(0, 1, 0, 3), "😃😄".to_owned()),
            span
        );
    }

    #[test]
    fn test_satifies() {
        let mut stream = CharStream::new("aBc");
        assert_eq!(true, stream.satifies(|c| c.is_lowercase()));
        stream.next();
        assert_eq!(false, stream.satifies(|c| c.is_lowercase()));
    }

    #[test]
    fn test_skip_rest_of_line() {
        let mut stream = CharStream::new("abc\ndef");
        stream.skip_rest_of_line();
        assert_eq!(Some('d'), stream.next());
        stream.skip_rest_of_line();
        assert_eq!(None, stream.next());
        stream.skip_rest_of_line();
        assert_eq!(None, stream.next());
    }

    #[test]
    fn test_seek() {
        let mut stream = CharStream::new("abc\ndefghi");
        let pos = Position::new(1, 2);
        stream.seek(pos);
        assert_eq!(Some('f'), stream.peek());
    }

    #[test]
    fn test_command_basic() {
        let mut stream = CharStream::new("\\foo@bar");
        let span = stream.command();
        assert_eq!(
            Span::new(Range::new_simple(0, 0, 0, 8), "\\foo@bar".to_owned()),
            span
        );
    }

    #[test]
    fn test_command_star() {
        let mut stream = CharStream::new("\\foo*");
        let span = stream.command();
        assert_eq!(
            Span::new(Range::new_simple(0, 0, 0, 5), "\\foo*".to_owned()),
            span
        );
    }

    #[test]
    fn test_command_escape() {
        let mut stream = CharStream::new("\\**");
        let span = stream.command();
        assert_eq!(
            Span::new(Range::new_simple(0, 0, 0, 2), "\\*".to_owned()),
            span
        );
    }
}
