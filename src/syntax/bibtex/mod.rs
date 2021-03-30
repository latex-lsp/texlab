mod kind;
mod lexer;

use std::marker::PhantomData;

use cstree::GreenNodeBuilder;

pub use self::kind::SyntaxKind::{self, *};
use self::lexer::Lexer;

use super::CstNode;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Lang {}

impl cstree::Language for Lang {
    type Kind = SyntaxKind;

    fn kind_from_raw(raw: cstree::SyntaxKind) -> Self::Kind {
        assert!(raw.0 <= ROOT as u16);
        unsafe { std::mem::transmute::<u16, SyntaxKind>(raw.0) }
    }

    fn kind_to_raw(kind: Self::Kind) -> cstree::SyntaxKind {
        kind.into()
    }
}

pub type SyntaxNode<D> = cstree::ResolvedNode<Lang, D>;

pub type SyntaxToken<D> = cstree::ResolvedToken<Lang, D>;

pub type SyntaxElement<D> = cstree::ResolvedElement<Lang, D>;

pub type SyntaxElementRef<'a, D> = cstree::ResolvedElementRef<'a, Lang, D>;

#[derive(Debug, Clone)]
pub struct Parse<D>
where
    D: 'static,
{
    pub root: SyntaxNode<D>,
}

struct Parser<'a, D> {
    lexer: Lexer<'a>,
    builder: GreenNodeBuilder<'static, 'static>,
    _phantom: PhantomData<D>,
}

impl<'a, D> Parser<'a, D> {
    pub fn new(lexer: Lexer<'a>) -> Self {
        Self {
            lexer,
            builder: GreenNodeBuilder::new(),
            _phantom: PhantomData::default(),
        }
    }

    fn consume(&mut self) {
        let (kind, text) = self.lexer.consume().unwrap();
        self.builder.token(kind.into(), text);
    }

    fn expect_or_missing(&mut self, kind: SyntaxKind) {
        if self.lexer.peek() == Some(kind) {
            self.consume();
            self.trivia();
        } else {
            self.builder.token(MISSING.into(), "");
        }
    }

    pub fn parse(mut self) -> Parse<D> {
        self.builder.start_node(ROOT.into());
        while let Some(kind) = self.lexer.peek() {
            match kind {
                PREAMBLE_TYPE => self.preamble(),
                STRING_TYPE => self.string(),
                COMMENT_TYPE => self.comment(),
                ENTRY_TYPE => self.entry(),
                _ => self.junk(),
            }
        }
        self.builder.finish_node();
        let (green, resolver) = self.builder.finish();
        Parse {
            root: SyntaxNode::new_root_with_resolver(green, resolver.unwrap()),
        }
    }

    fn trivia(&mut self) {
        while self.lexer.peek() == Some(WHITESPACE) {
            self.consume();
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
            self.consume();
        }
        self.builder.finish_node();
    }

    fn left_delimiter_or_missing(&mut self) {
        if self
            .lexer
            .peek()
            .filter(|&kind| matches!(kind, L_BRACE | L_PAREN))
            .is_some()
        {
            self.consume();
            self.trivia();
        } else {
            self.builder.token(MISSING.into(), "");
        }
    }

    fn right_delimiter_or_missing(&mut self) {
        if self
            .lexer
            .peek()
            .filter(|&kind| matches!(kind, R_BRACE | R_PAREN))
            .is_some()
        {
            self.consume();
            self.trivia();
        } else {
            self.builder.token(MISSING.into(), "");
        }
    }

    fn value_or_missing(&mut self) {
        if self
            .lexer
            .peek()
            .filter(|&kind| matches!(kind, L_BRACE | QUOTE | NUMBER | WORD))
            .is_some()
        {
            self.value();
        } else {
            self.builder.token(MISSING.into(), "");
        }
    }

    fn preamble(&mut self) {
        self.builder.start_node(PREAMBLE.into());
        self.consume();
        self.trivia();
        self.left_delimiter_or_missing();
        self.value_or_missing();
        self.right_delimiter_or_missing();
    }

    fn string(&mut self) {
        self.builder.start_node(STRING.into());
        self.consume();
        self.trivia();
        self.left_delimiter_or_missing();

        if self.lexer.peek() != Some(WORD) {
            self.builder.token(MISSING.into(), "");
            self.builder.finish_node();
            return;
        }
        self.consume();
        self.trivia();
        self.expect_or_missing(EQUALITY_SIGN);

        self.value_or_missing();

        self.right_delimiter_or_missing();
        self.builder.finish_node();
    }

    fn comment(&mut self) {
        self.builder.start_node(COMMENT.into());
        self.consume();
        self.builder.finish_node();
    }

    fn entry(&mut self) {
        self.builder.start_node(ENTRY.into());
        self.consume();
        self.trivia();

        self.left_delimiter_or_missing();

        if self.lexer.peek() != Some(WORD) {
            self.builder.token(MISSING.into(), "");
            self.builder.finish_node();
            return;
        }
        self.consume();

        while let Some(kind) = self.lexer.peek() {
            match kind {
                WHITESPACE => self.consume(),
                WORD => self.field(),
                COMMA => self.consume(),
                _ => break,
            };
        }

        self.right_delimiter_or_missing();

        self.builder.finish_node();
    }

    fn field(&mut self) {
        self.builder.start_node(FIELD.into());
        self.consume();
        self.trivia();

        if self.lexer.peek() == Some(EQUALITY_SIGN) {
            self.consume();
            self.trivia();
        } else {
            self.builder.token(MISSING.into(), "");
        }

        if self
            .lexer
            .peek()
            .filter(|&kind| matches!(kind, L_BRACE | QUOTE | NUMBER | WORD))
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
        while let Some(kind) = self.lexer.peek() {
            match kind {
                WHITESPACE => self.consume(),
                L_BRACE | QUOTE | NUMBER | WORD => self.token(),
                HASH => self.consume(),
                _ => break,
            }
        }
        self.builder.finish_node();
    }

    fn token(&mut self) {
        self.builder.start_node(TOKEN.into());
        match self.lexer.peek().unwrap() {
            L_BRACE => self.brace_group(),
            QUOTE => self.quote_group(),
            NUMBER => self.consume(),
            WORD => self.consume(),
            _ => unreachable!(),
        };
        self.builder.finish_node();
    }

    fn brace_group(&mut self) {
        self.builder.start_node(BRACE_GROUP.into());
        self.consume();

        while let Some(kind) = self.lexer.peek() {
            match kind {
                WHITESPACE => self.consume(),
                PREAMBLE_TYPE => break,
                STRING_TYPE => break,
                COMMENT_TYPE => break,
                ENTRY_TYPE => break,
                WORD => self.consume(),
                L_BRACE => self.brace_group(),
                R_BRACE => break,
                L_PAREN => self.consume(),
                R_PAREN => self.consume(),
                COMMA => self.consume(),
                HASH => self.consume(),
                QUOTE => self.consume(),
                EQUALITY_SIGN => self.consume(),
                NUMBER => self.consume(),
                COMMAND_NAME => self.consume(),
                _ => unreachable!(),
            };
        }

        self.expect_or_missing(R_BRACE);

        self.builder.finish_node();
    }

    fn quote_group(&mut self) {
        self.builder.start_node(QUOTE_GROUP.into());
        self.consume();

        while let Some(kind) = self.lexer.peek() {
            match kind {
                WHITESPACE => self.consume(),
                PREAMBLE_TYPE => break,
                STRING_TYPE => break,
                COMMENT_TYPE => break,
                ENTRY_TYPE => break,
                WORD => self.consume(),
                L_BRACE => self.brace_group(),
                R_BRACE => break,
                L_PAREN => self.consume(),
                R_PAREN => self.consume(),
                COMMA => self.consume(),
                HASH => self.consume(),
                QUOTE => break,
                EQUALITY_SIGN => self.consume(),
                NUMBER => self.consume(),
                COMMAND_NAME => self.consume(),
                _ => unreachable!(),
            };
        }

        self.expect_or_missing(QUOTE);
        self.builder.finish_node();
    }
}

pub fn parse<D>(text: &str) -> Parse<D>
where
    D: 'static,
{
    Parser::new(Lexer::new(text)).parse()
}

macro_rules! cst_node {
    ($name:ident, $($kind:pat),+) => {
        #[derive(Clone)]
        #[repr(transparent)]
        pub struct $name<'a, D: 'static>(&'a SyntaxNode<D>);

        impl<'a, D> CstNode<'a, D> for $name<'a, D> {
            type Lang = Lang;

            fn cast(node: &'a cstree::ResolvedNode<Self::Lang, D>) -> Option<Self>
            where
                Self: Sized,
            {
                match node.kind() {
                    $($kind => Some(Self(node)),)+
                    _ => None,
                }
            }

            fn syntax(&self) -> &'a cstree::ResolvedNode<Self::Lang, D> {
                &self.0
            }
        }
    };
}

pub trait HasBraces<'a, D: 'static>: CstNode<'a, D, Lang = Lang> {
    fn left_brace(&self) -> Option<&'a SyntaxToken<D>> {
        self.syntax()
            .children_with_tokens()
            .filter_map(|node| node.into_token())
            .find(|node| node.kind() == L_BRACE.into())
    }

    fn right_brace(&self) -> Option<&'a SyntaxToken<D>> {
        self.syntax()
            .children_with_tokens()
            .filter_map(|node| node.into_token())
            .find(|node| node.kind() == R_BRACE.into())
    }
}

pub trait HasQuotes<'a, D: 'static>: CstNode<'a, D, Lang = Lang> {
    fn left_quote(&self) -> Option<&'a SyntaxToken<D>> {
        self.syntax()
            .children_with_tokens()
            .filter_map(|node| node.into_token())
            .find(|node| node.kind() == QUOTE.into())
    }

    fn right_quote(&self) -> Option<&'a SyntaxToken<D>> {
        self.syntax()
            .children_with_tokens()
            .filter_map(|node| node.into_token())
            .filter(|node| node.kind() == QUOTE.into())
            .skip(1)
            .next()
    }
}

pub trait HasDelimiters<'a, D: 'static>: CstNode<'a, D, Lang = Lang> {
    fn left_delimiter(&self) -> Option<&'a SyntaxToken<D>> {
        self.syntax()
            .children_with_tokens()
            .filter_map(|node| node.into_token())
            .find(|node| matches!(node.kind(), L_BRACE | L_PAREN))
    }

    fn right_delimiter(&self) -> Option<&'a SyntaxToken<D>> {
        self.syntax()
            .children_with_tokens()
            .filter_map(|node| node.into_token())
            .find(|node| matches!(node.kind(), R_BRACE | R_PAREN))
    }
}

pub trait HasType<'a, D: 'static>: CstNode<'a, D, Lang = Lang> {
    fn ty(&self) -> Option<&'a SyntaxToken<D>> {
        self.syntax()
            .children_with_tokens()
            .filter_map(|node| node.into_token())
            .find(|node| {
                matches!(
                    node.kind(),
                    PREAMBLE_TYPE | STRING_TYPE | COMMENT_TYPE | ENTRY_TYPE
                )
            })
    }
}

cst_node!(Root, ROOT);

cst_node!(Junk, JUNK);

cst_node!(Comment, COMMENT);

impl<'a, D: 'static> HasType<'a, D> for Comment<'a, D> {}

cst_node!(Preamble, PREAMBLE);

impl<'a, D: 'static> HasType<'a, D> for Preamble<'a, D> {}

impl<'a, D: 'static> HasDelimiters<'a, D> for Preamble<'a, D> {}

impl<'a, D: 'static> Preamble<'a, D> {
    pub fn value(&self) -> Option<Value<'a, D>> {
        self.syntax().children().find_map(Value::cast)
    }
}

cst_node!(String, STRING);

impl<'a, D: 'static> HasType<'a, D> for String<'a, D> {}

impl<'a, D: 'static> HasDelimiters<'a, D> for String<'a, D> {}

impl<'a, D: 'static> String<'a, D> {
    pub fn value(&self) -> Option<Value<'a, D>> {
        self.syntax().children().find_map(Value::cast)
    }
}

cst_node!(Entry, ENTRY);

impl<'a, D> HasType<'a, D> for Entry<'a, D> {}

impl<'a, D> HasDelimiters<'a, D> for Entry<'a, D> {}

impl<'a, D> Entry<'a, D> {
    pub fn key(&self) -> Option<&'a SyntaxToken<D>> {
        self.syntax()
            .children_with_tokens()
            .filter_map(|node| node.into_token())
            .find(|node| node.kind() == WORD)
    }

    pub fn fields(&self) -> impl Iterator<Item = Field<'a, D>> {
        self.syntax().children().filter_map(Field::cast)
    }
}

cst_node!(Field, FIELD);

impl<'a, D> Field<'a, D> {
    pub fn name(&self) -> Option<&'a SyntaxToken<D>> {
        self.syntax()
            .children_with_tokens()
            .filter_map(|node| node.into_token())
            .find(|node| node.kind() == WORD)
    }

    pub fn value(&self) -> Option<Value<'a, D>> {
        self.syntax().children().find_map(Value::cast)
    }
}

cst_node!(Value, VALUE);

impl<'a, D> Value<'a, D> {
    pub fn tokens(&self) -> impl Iterator<Item = Token<'a, D>> {
        self.syntax().children().filter_map(Token::cast)
    }
}

cst_node!(Token, TOKEN);

cst_node!(BraceGroup, BRACE_GROUP);

impl<'a, D> HasBraces<'a, D> for BraceGroup<'a, D> {}

cst_node!(QuoteGroup, QUOTE_GROUP);

impl<'a, D> HasQuotes<'a, D> for QuoteGroup<'a, D> {}

#[cfg(test)]
mod tests {
    use insta::assert_debug_snapshot;

    use super::*;

    fn setup(text: &str) -> SyntaxNode<()> {
        parse(&text.trim().replace("\r", "")).root
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
