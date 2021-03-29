use std::mem;

use logos::Logos;
use rowan::{GreenNode, GreenNodeBuilder};

pub use self::SyntaxKind::*;

use super::AstNode;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, PartialOrd, Ord)]
#[allow(non_camel_case_types)]
#[repr(u16)]
pub enum SyntaxKind {
    ERROR = 0,
    MISSING,

    WHITESPACE,
    PREAMBLE_TYPE,
    STRING_TYPE,
    COMMENT_TYPE,
    ENTRY_TYPE,
    WORD,
    L_BRACE,
    R_BRACE,
    L_PAREN,
    R_PAREN,
    COMMA,
    HASH,
    QUOTE,
    EQUALITY_SIGN,
    NUMBER,
    COMMAND_NAME,

    JUNK,
    PREAMBLE,
    STRING,
    COMMENT,
    ENTRY,
    FIELD,
    VALUE,
    TOKEN,
    BRACE_GROUP,
    QUOTE_GROUP,
    ROOT,
}

impl From<SyntaxKind> for rowan::SyntaxKind {
    fn from(kind: SyntaxKind) -> Self {
        Self(kind as u16)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Lang {}

impl rowan::Language for Lang {
    type Kind = SyntaxKind;

    fn kind_from_raw(raw: rowan::SyntaxKind) -> Self::Kind {
        assert!(raw.0 <= ROOT as u16);
        unsafe { mem::transmute::<u16, SyntaxKind>(raw.0) }
    }

    fn kind_to_raw(kind: Self::Kind) -> rowan::SyntaxKind {
        kind.into()
    }
}

pub struct Parse {
    pub green_node: GreenNode,
}

pub type SyntaxNode = rowan::SyntaxNode<Lang>;

pub type SyntaxToken = rowan::SyntaxToken<Lang>;

pub type SyntaxElement = rowan::NodeOrToken<SyntaxNode, SyntaxToken>;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, PartialOrd, Ord, Logos)]
#[allow(non_camel_case_types)]
#[repr(u16)]
enum LogosToken {
    #[regex(r"\s+")]
    WHITESPACE = 2,

    #[regex(r"@[Pp][Rr][Ee][Aa][Mm][Bb][Ll][Ee]")]
    PREAMBLE_TYPE,

    #[regex(r"@[Ss][Tt][Rr][Ii][Nn][Gg]")]
    STRING_TYPE,

    #[regex(r"@[Cc][Oo][Mm][Mm][Ee][Nn][Tt]")]
    COMMENT_TYPE,

    #[regex(r"@[!\$\&\*\+\-\./:;<>\?@\[\]\\\^_`\|\~a-zA-Z][!\$\&\*\+\-\./:;<>\?@\[\]\\\^_`\|\~a-zA-Z0-9]*")]
    ENTRY_TYPE,

    #[regex(r#"[^\s\{\}\(\),#"=\d\\]+"#)]
    #[error]
    WORD,

    #[token("{")]
    L_BRACE,

    #[token("}")]
    R_BRACE,

    #[token("(")]
    L_PAREN,

    #[token(")")]
    R_PAREN,

    #[token(",")]
    COMMA,

    #[token("#")]
    HASH,

    #[token("\"")]
    QUOTE,

    #[token("=")]
    EQUALITY_SIGN,

    #[regex(r"\d+")]
    NUMBER,

    #[regex(r"\\([^\r\n]|[@a-zA-Z]+\*?)?")]
    COMMAND_NAME,
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct TokenSource<'a> {
    tokens: Vec<(SyntaxKind, &'a str)>,
}

impl<'a> TokenSource<'a> {
    pub fn new(text: &'a str) -> Self {
        let mut tokens = Vec::new();
        let mut lexer = LogosToken::lexer(text);
        while let Some(kind) = lexer.next() {
            tokens.push((
                unsafe { mem::transmute::<LogosToken, SyntaxKind>(kind) },
                lexer.slice(),
            ));
        }
        tokens.reverse();
        Self { tokens }
    }

    pub fn peek(&self) -> Option<SyntaxKind> {
        self.tokens.last().map(|(kind, _)| *kind)
    }

    pub fn consume(&mut self) -> Option<(SyntaxKind, &'a str)> {
        self.tokens.pop()
    }
}

struct Parser<'a> {
    tokens: TokenSource<'a>,
    builder: GreenNodeBuilder<'a>,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: TokenSource<'a>) -> Self {
        Self {
            tokens,
            builder: GreenNodeBuilder::new(),
        }
    }

    fn consume(&mut self) {
        let (kind, text) = self.tokens.consume().unwrap();
        self.builder.token(kind.into(), text);
    }

    fn expect_or_missing(&mut self, kind: SyntaxKind) {
        if self.tokens.peek() == Some(kind) {
            self.consume();
            self.trivia();
        } else {
            self.builder.token(MISSING.into(), "");
        }
    }

    pub fn parse(mut self) -> Parse {
        self.builder.start_node(ROOT.into());
        while let Some(kind) = self.tokens.peek() {
            match kind {
                PREAMBLE_TYPE => self.preamble(),
                STRING_TYPE => self.string(),
                COMMENT_TYPE => self.comment(),
                ENTRY_TYPE => self.entry(),
                _ => self.junk(),
            }
        }
        self.builder.finish_node();
        let green_node = self.builder.finish();
        Parse { green_node }
    }

    fn trivia(&mut self) {
        while self.tokens.peek() == Some(WHITESPACE) {
            self.consume();
        }
    }

    fn junk(&mut self) {
        self.builder.start_node(JUNK.into());
        while self
            .tokens
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
            .tokens
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
            .tokens
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
            .tokens
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

        if self.tokens.peek() != Some(WORD) {
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

        if self.tokens.peek() != Some(WORD) {
            self.builder.token(MISSING.into(), "");
            self.builder.finish_node();
            return;
        }
        self.consume();

        while let Some(kind) = self.tokens.peek() {
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

        if self.tokens.peek() == Some(EQUALITY_SIGN) {
            self.consume();
            self.trivia();
        } else {
            self.builder.token(MISSING.into(), "");
        }

        if self
            .tokens
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
        while let Some(kind) = self.tokens.peek() {
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
        match self.tokens.peek().unwrap() {
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

        while let Some(kind) = self.tokens.peek() {
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

        while let Some(kind) = self.tokens.peek() {
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

pub fn parse(text: &str) -> Parse {
    Parser::new(TokenSource::new(text)).parse()
}

macro_rules! ast_node {
    ($name:ident, $($kind:pat),+) => {
        #[derive(Debug, Clone)]
        #[repr(transparent)]
        pub struct $name(SyntaxNode);

        impl AstNode for $name {
            type Lang = Lang;

            fn cast(node: rowan::SyntaxNode<Self::Lang>) -> Option<Self>
            where
                Self: Sized,
            {
                match node.kind() {
                    $($kind => Some(Self(node)),)+
                    _ => None,
                }
            }

            fn syntax(&self) -> &rowan::SyntaxNode<Self::Lang> {
                &self.0
            }
        }
    };
}

pub trait HasBraces: AstNode<Lang = Lang> {
    fn left_brace(&self) -> Option<SyntaxToken> {
        self.syntax()
            .children_with_tokens()
            .filter_map(|node| node.into_token())
            .find(|node| node.kind() == L_BRACE.into())
    }

    fn right_brace(&self) -> Option<SyntaxToken> {
        self.syntax()
            .children_with_tokens()
            .filter_map(|node| node.into_token())
            .find(|node| node.kind() == R_BRACE.into())
    }
}

pub trait HasQuotes: AstNode<Lang = Lang> {
    fn left_quote(&self) -> Option<SyntaxToken> {
        self.syntax()
            .children_with_tokens()
            .filter_map(|node| node.into_token())
            .find(|node| node.kind() == QUOTE.into())
    }

    fn right_quote(&self) -> Option<SyntaxToken> {
        self.syntax()
            .children_with_tokens()
            .filter_map(|node| node.into_token())
            .filter(|node| node.kind() == QUOTE.into())
            .skip(1)
            .next()
    }
}

pub trait HasDelimiters: AstNode<Lang = Lang> {
    fn left_delimiter(&self) -> Option<SyntaxToken> {
        self.syntax()
            .children_with_tokens()
            .filter_map(|node| node.into_token())
            .find(|node| matches!(node.kind(), L_BRACE | L_PAREN))
    }

    fn right_delimiter(&self) -> Option<SyntaxToken> {
        self.syntax()
            .children_with_tokens()
            .filter_map(|node| node.into_token())
            .find(|node| matches!(node.kind(), R_BRACE | R_PAREN))
    }
}

pub trait HasType: AstNode<Lang = Lang> {
    fn ty(&self) -> Option<SyntaxToken> {
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

ast_node!(Root, ROOT);

ast_node!(Junk, JUNK);

ast_node!(Comment, COMMENT);

impl HasType for Comment {}

ast_node!(Preamble, PREAMBLE);

impl HasType for Preamble {}

impl HasDelimiters for Preamble {}

impl Preamble {
    pub fn value(&self) -> Option<Value> {
        self.syntax().children().find_map(Value::cast)
    }
}

ast_node!(String, STRING);

impl HasType for String {}

impl HasDelimiters for String {}

impl String {
    pub fn value(&self) -> Option<Value> {
        self.syntax().children().find_map(Value::cast)
    }
}

ast_node!(Entry, ENTRY);

impl HasType for Entry {}

impl HasDelimiters for Entry {}

impl Entry {
    pub fn key(&self) -> Option<SyntaxToken> {
        self.syntax()
            .children_with_tokens()
            .filter_map(|node| node.into_token())
            .find(|node| node.kind() == WORD)
    }

    pub fn fields(&self) -> impl Iterator<Item = Field> {
        self.syntax().children().filter_map(Field::cast)
    }
}

ast_node!(Field, FIELD);

impl Field {
    pub fn name(&self) -> Option<SyntaxToken> {
        self.syntax()
            .children_with_tokens()
            .filter_map(|node| node.into_token())
            .find(|node| node.kind() == WORD)
    }

    pub fn value(&self) -> Option<Value> {
        self.syntax().children().find_map(Value::cast)
    }
}

ast_node!(Value, VALUE);

impl Value {
    pub fn tokens(&self) -> impl Iterator<Item = Token> {
        self.syntax().children().filter_map(Token::cast)
    }
}

ast_node!(Token, TOKEN);

ast_node!(BraceGroup, BRACE_GROUP);

impl HasBraces for BraceGroup {}

ast_node!(QuoteGroup, QUOTE_GROUP);

impl HasQuotes for QuoteGroup {}

#[cfg(test)]
mod tests {
    use insta::assert_debug_snapshot;

    use super::*;

    fn setup(text: &str) -> SyntaxNode {
        SyntaxNode::new_root(parse(&text.trim().replace("\r", "")).green_node)
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
