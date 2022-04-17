use rowan::{GreenNode, GreenNodeBuilder};

use super::{
    lexer::Lexer,
    SyntaxKind::{self, *},
};

#[derive(Debug, Clone)]
pub struct Parse {
    pub green: GreenNode,
}

struct Parser<'a> {
    lexer: Lexer<'a>,
    builder: GreenNodeBuilder<'static>,
}

impl<'a> Parser<'a> {
    pub fn new(lexer: Lexer<'a>) -> Self {
        Self {
            lexer,
            builder: GreenNodeBuilder::new(),
        }
    }

    fn peek(&self) -> Option<SyntaxKind> {
        self.lexer.peek()
    }

    fn eat(&mut self) {
        let (kind, text) = self.lexer.consume().unwrap();
        self.builder.token(kind.into(), text);
    }

    fn expect(&mut self, kind: SyntaxKind) {
        if self.peek() == Some(kind) {
            self.eat();
            self.trivia();
        } else {
            self.builder.token(MISSING.into(), "");
        }
    }

    pub fn parse(mut self) -> Parse {
        self.builder.start_node(ROOT.into());
        while let Some(kind) = self.peek() {
            match kind {
                PREAMBLE_TYPE => self.preamble(),
                STRING_TYPE => self.string(),
                COMMENT_TYPE => self.comment(),
                ENTRY_TYPE => self.entry(),
                _ => self.junk(),
            }
        }
        self.builder.finish_node();
        let green = self.builder.finish();
        Parse { green }
    }

    fn trivia(&mut self) {
        while self.peek() == Some(WHITESPACE) {
            self.eat();
        }
    }

    fn junk(&mut self) {
        self.builder.start_node(JUNK.into());
        while self
            .lexer
            .peek()
            .filter(|&kind| {
                !matches!(
                    kind,
                    PREAMBLE_TYPE | STRING_TYPE | COMMENT_TYPE | ENTRY_TYPE,
                )
            })
            .is_some()
        {
            self.eat();
        }
        self.builder.finish_node();
    }

    fn left_delimiter_or_missing(&mut self) {
        if self
            .lexer
            .peek()
            .filter(|&kind| matches!(kind, L_CURLY | L_PAREN))
            .is_some()
        {
            self.eat();
            self.trivia();
        } else {
            self.builder.token(MISSING.into(), "");
        }
    }

    fn right_delimiter_or_missing(&mut self) {
        if self
            .lexer
            .peek()
            .filter(|&kind| matches!(kind, R_CURLY | R_PAREN))
            .is_some()
        {
            self.eat();
            self.trivia();
        } else {
            self.builder.token(MISSING.into(), "");
        }
    }

    fn value_or_missing(&mut self) {
        if self
            .lexer
            .peek()
            .filter(|&kind| matches!(kind, L_CURLY | QUOTE | WORD))
            .is_some()
        {
            self.value();
        } else {
            self.builder.token(MISSING.into(), "");
        }
    }

    fn preamble(&mut self) {
        self.builder.start_node(PREAMBLE.into());
        self.eat();
        self.trivia();
        self.left_delimiter_or_missing();
        self.value_or_missing();
        self.right_delimiter_or_missing();
        self.builder.finish_node();
    }

    fn string(&mut self) {
        self.builder.start_node(STRING.into());
        self.eat();
        self.trivia();
        self.left_delimiter_or_missing();

        if self.peek() != Some(WORD) {
            self.builder.token(MISSING.into(), "");
            self.builder.finish_node();
            return;
        }
        self.eat();
        self.trivia();
        self.expect(EQUALITY_SIGN);

        self.value_or_missing();

        self.right_delimiter_or_missing();
        self.builder.finish_node();
    }

    fn comment(&mut self) {
        self.builder.start_node(COMMENT.into());
        self.eat();
        self.builder.finish_node();
    }

    fn entry(&mut self) {
        self.builder.start_node(ENTRY.into());
        self.eat();
        self.trivia();

        self.left_delimiter_or_missing();

        if self.peek() != Some(WORD) {
            self.builder.token(MISSING.into(), "");
            self.builder.finish_node();
            return;
        }
        self.key();

        while let Some(kind) = self.peek() {
            match kind {
                WHITESPACE => self.eat(),
                WORD => self.field(),
                COMMA => self.eat(),
                _ => break,
            };
        }

        self.right_delimiter_or_missing();

        self.builder.finish_node();
    }

    fn key(&mut self) {
        self.builder.start_node(KEY.into());
        while self
            .peek()
            .filter(|&kind| matches!(kind, WORD | WHITESPACE))
            .is_some()
        {
            self.eat();
        }
        self.builder.finish_node();
    }

    fn field(&mut self) {
        self.builder.start_node(FIELD.into());
        self.eat();
        self.trivia();

        if self.peek() == Some(EQUALITY_SIGN) {
            self.eat();
            self.trivia();
        } else {
            self.builder.token(MISSING.into(), "");
        }

        if self
            .lexer
            .peek()
            .filter(|&kind| matches!(kind, L_CURLY | QUOTE | WORD))
            .is_some()
        {
            self.value();
        } else {
            self.builder.token(MISSING.into(), "");
        }

        self.builder.finish_node();
    }

    fn value(&mut self) {
        self.builder.start_node(VALUE.into());
        self.token();
        while let Some(kind) = self.peek() {
            match kind {
                WHITESPACE => self.eat(),
                L_CURLY | QUOTE | WORD => self.token(),
                HASH => self.eat(),
                _ => break,
            }
        }
        self.builder.finish_node();
    }

    fn token(&mut self) {
        self.builder.start_node(TOKEN.into());
        match self.peek().unwrap() {
            L_CURLY => self.brace_group(),
            QUOTE => self.quote_group(),
            WORD => self.eat(),
            _ => unreachable!(),
        };
        self.builder.finish_node();
    }

    fn brace_group(&mut self) {
        self.builder.start_node(BRACE_GROUP.into());
        self.eat();

        while let Some(kind) = self.peek() {
            match kind {
                WHITESPACE => self.eat(),
                PREAMBLE_TYPE => break,
                STRING_TYPE => break,
                COMMENT_TYPE => break,
                ENTRY_TYPE => break,
                WORD => self.eat(),
                L_CURLY => self.brace_group(),
                R_CURLY => break,
                L_PAREN => self.eat(),
                R_PAREN => self.eat(),
                COMMA => self.eat(),
                HASH => self.eat(),
                QUOTE => self.eat(),
                EQUALITY_SIGN => self.eat(),
                COMMAND_NAME => self.eat(),
                _ => unreachable!(),
            };
        }

        self.expect(R_CURLY);

        self.builder.finish_node();
    }

    fn quote_group(&mut self) {
        self.builder.start_node(QUOTE_GROUP.into());
        self.eat();

        while let Some(kind) = self.peek() {
            match kind {
                WHITESPACE => self.eat(),
                PREAMBLE_TYPE => break,
                STRING_TYPE => break,
                COMMENT_TYPE => break,
                ENTRY_TYPE => break,
                WORD => self.eat(),
                L_CURLY => self.brace_group(),
                R_CURLY => break,
                L_PAREN => self.eat(),
                R_PAREN => self.eat(),
                COMMA => self.eat(),
                HASH => self.eat(),
                QUOTE => break,
                EQUALITY_SIGN => self.eat(),
                COMMAND_NAME => self.eat(),
                _ => unreachable!(),
            };
        }

        self.expect(QUOTE);
        self.builder.finish_node();
    }
}

pub fn parse(text: &str) -> Parse {
    Parser::new(Lexer::new(text)).parse()
}

#[cfg(test)]
mod tests {
    use insta::assert_debug_snapshot;

    use crate::syntax::bibtex;

    use super::*;

    fn setup(text: &str) -> bibtex::SyntaxNode {
        bibtex::SyntaxNode::new_root(parse(&text.trim().replace('\r', "")).green)
    }

    #[test]
    fn test_empty() {
        assert_debug_snapshot!(setup(r#""#));
    }

    #[test]
    fn test_junk() {
        assert_debug_snapshot!(setup(r#"Hello World!"#));
    }

    #[test]
    fn test_preamble_complete() {
        assert_debug_snapshot!(setup(r#"@preamble{ "Hello World" }"#));
    }

    #[test]
    fn test_preamble_missing_end() {
        assert_debug_snapshot!(setup(r#"@preamble{ "Hello World" "#));
    }

    #[test]
    fn test_preamble_casing() {
        assert_debug_snapshot!(setup(r#"@preAmbLe{ "Hello World" }"#));
    }

    #[test]
    fn test_string_complete() {
        assert_debug_snapshot!(setup(r#"@string{foo = {Hello World}}"#));
    }

    #[test]
    fn test_string_incomplete() {
        assert_debug_snapshot!(setup(r#"@string{foo = {Hello World}"#));
    }

    #[test]
    fn test_string_concatenation() {
        assert_debug_snapshot!(setup(
            r#"@string{foo = {Hello World}} @string{bar = foo # "!"}"#
        ));
    }

    #[test]
    fn test_string_casing() {
        assert_debug_snapshot!(setup(r#"@STRING{foo = "Hello World"}"#));
    }

    #[test]
    fn test_string_missing_quote() {
        assert_debug_snapshot!(setup(r#"@STRING{foo = "Hello World}"#));
    }

    #[test]
    fn test_entry_no_fields() {
        assert_debug_snapshot!(setup(r#"@article{foo,}"#));
    }

    #[test]
    fn test_entry_no_fields_missing_comma() {
        assert_debug_snapshot!(setup(r#"@article{foo}"#));
    }

    #[test]
    fn test_entry_one_field() {
        assert_debug_snapshot!(setup(r#"@article{foo, author = {Foo Bar}}"#));
    }

    #[test]
    fn test_entry_one_field_number_key() {
        assert_debug_snapshot!(setup(r#"@article{foo2021, author = {Foo Bar}}"#));
    }

    #[test]
    fn test_entry_one_field_trailing_comma() {
        assert_debug_snapshot!(setup(r#"@article{foo, author = {Foo Bar},}"#));
    }

    #[test]
    fn test_entry_two_fields() {
        assert_debug_snapshot!(setup(
            r#"@article{foo, author = {Foo Bar}, title = {Hello World}}"#
        ));
    }

    #[test]
    fn test_entry_two_fields_incomplete() {
        assert_debug_snapshot!(setup(r#"@article{foo, author = {Foo Bar}, t}"#));
    }

    #[test]
    fn test_entry_complete_parens() {
        assert_debug_snapshot!(setup(
            r#"@article(foo, author = {Foo Bar}, title = {Hello})"#
        ));
    }
}
