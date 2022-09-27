use logos::Logos;
use rowan::{GreenNode, GreenNodeBuilder};

use crate::syntax::bibtex::SyntaxKind::{self, *};

pub fn parse_bibtex(input: &str) -> GreenNode {
    let mut ptr = TokenPtr {
        builder: GreenNodeBuilder::new(),
        lexer: RootToken::lexer(input),
        token: None,
    };

    ptr.builder.start_node(ROOT.into());

    while let Some(token) = ptr.current() {
        match token {
            RootToken::Preamble => ptr = preamble(ptr),
            RootToken::String => ptr = string(ptr),
            RootToken::Entry => ptr = entry(ptr),
            RootToken::Comment | RootToken::Junk => ptr.bump(),
        }
    }

    ptr.builder.finish_node();
    ptr.builder.finish()
}

fn preamble(mut ptr: TokenPtr<RootToken>) -> TokenPtr<RootToken> {
    ptr.builder.start_node(PREAMBLE.into());
    ptr.bump();
    let mut ptr = ptr.morph();
    ptr.expect(BodyToken::Whitespace);
    ptr.expect(BodyToken::LDelim);
    ptr.expect(BodyToken::Whitespace);
    ptr = value(ptr.morph()).morph();
    ptr.expect(BodyToken::RDelim);
    ptr.builder.finish_node();
    ptr.morph()
}

fn string(mut ptr: TokenPtr<RootToken>) -> TokenPtr<RootToken> {
    ptr.builder.start_node(STRING.into());
    ptr.bump();
    let mut ptr = ptr.morph();
    ptr.expect(BodyToken::Whitespace);
    ptr.expect(BodyToken::LDelim);
    ptr.expect(BodyToken::Whitespace);
    ptr.expect(BodyToken::Name);
    ptr.expect(BodyToken::Whitespace);
    ptr.expect(BodyToken::Eq);
    ptr.expect(BodyToken::Whitespace);
    ptr = value(ptr.morph()).morph();
    ptr.expect(BodyToken::RDelim);
    ptr.builder.finish_node();
    ptr.morph()
}

fn entry(mut ptr: TokenPtr<RootToken>) -> TokenPtr<RootToken> {
    ptr.builder.start_node(ENTRY.into());
    ptr.bump();
    let mut ptr = ptr.morph();
    ptr.expect(BodyToken::Whitespace);
    ptr.expect(BodyToken::LDelim);
    ptr.expect(BodyToken::Whitespace);
    ptr.expect(BodyToken::Name);
    ptr.expect(BodyToken::Whitespace);
    ptr.expect(BodyToken::Comma);
    ptr.expect(BodyToken::Whitespace);

    while ptr.at(BodyToken::Name) {
        ptr = field(ptr);
        ptr.expect(BodyToken::Whitespace);
    }

    ptr.expect(BodyToken::RDelim);
    ptr.builder.finish_node();
    ptr.morph()
}

fn field(mut ptr: TokenPtr<BodyToken>) -> TokenPtr<BodyToken> {
    ptr.builder.start_node(FIELD.into());
    ptr.bump();
    ptr.expect(BodyToken::Whitespace);
    ptr.expect(BodyToken::Eq);
    ptr.expect(BodyToken::Whitespace);
    ptr = value(ptr.morph()).morph();
    ptr.expect(BodyToken::Whitespace);
    ptr.expect(BodyToken::Comma);
    ptr.builder.finish_node();
    ptr
}

fn value(mut ptr: TokenPtr<ValueToken>) -> TokenPtr<ValueToken> {
    let checkpoint = ptr.builder.checkpoint();
    if let Some(token) = ptr.current() {
        match token {
            ValueToken::Whitespace => unreachable!(),
            ValueToken::Pound | ValueToken::Comma | ValueToken::RCurly => return ptr,
            ValueToken::Integer | ValueToken::Name => ptr = literal(ptr),
            ValueToken::LCurly => ptr = curly_group(ptr.morph()).morph(),
            ValueToken::Quote => ptr = quote_group(ptr.morph()).morph(),
        };

        ptr.expect(ValueToken::Whitespace);
        if ptr.at(ValueToken::Pound) {
            ptr.builder.start_node_at(checkpoint, JOIN.into());
            ptr.bump();
            ptr.expect(ValueToken::Whitespace);
            ptr = value(ptr);
            ptr.builder.finish_node();
        }
    }

    ptr
}

fn literal(mut ptr: TokenPtr<ValueToken>) -> TokenPtr<ValueToken> {
    ptr.builder.start_node(LITERAL.into());
    ptr.bump();
    ptr.builder.finish_node();
    ptr
}

fn curly_group(mut ptr: TokenPtr<ContentToken>) -> TokenPtr<ContentToken> {
    ptr.builder.start_node(CURLY_GROUP.into());
    ptr.bump();
    ptr.expect(ContentToken::Whitespace);

    while let Some(token) = ptr.current() {
        match token {
            ContentToken::RCurly => break,
            ContentToken::Whitespace
            | ContentToken::Nbsp
            | ContentToken::Comma
            | ContentToken::Integer
            | ContentToken::Word => ptr.bump(),
            ContentToken::LCurly => ptr = curly_group(ptr),
            ContentToken::Quote => ptr = quote_group(ptr),
            ContentToken::AccentName => ptr = accent(ptr),
            ContentToken::CommandName => ptr = command(ptr),
        };
    }

    ptr.expect(ContentToken::RCurly);
    ptr.builder.finish_node();
    ptr
}

fn quote_group(mut ptr: TokenPtr<ContentToken>) -> TokenPtr<ContentToken> {
    ptr.builder.start_node(QUOTE_GROUP.into());
    ptr.bump();
    ptr.expect(ContentToken::Whitespace);

    while let Some(token) = ptr.current() {
        match token {
            ContentToken::Quote => break,
            ContentToken::Whitespace
            | ContentToken::Nbsp
            | ContentToken::Comma
            | ContentToken::RCurly
            | ContentToken::Integer
            | ContentToken::Word => ptr.bump(),
            ContentToken::LCurly => ptr = curly_group(ptr),
            ContentToken::AccentName => ptr = accent(ptr),
            ContentToken::CommandName => ptr = command(ptr),
        };
    }

    ptr.expect(ContentToken::Quote);
    ptr.builder.finish_node();
    ptr
}

fn accent(mut ptr: TokenPtr<ContentToken>) -> TokenPtr<ContentToken> {
    ptr.builder.start_node(ACCENT.into());
    ptr.bump();
    ptr.expect(ContentToken::Whitespace);

    let group = ptr.at(ContentToken::LCurly);
    if group {
        ptr.expect(ContentToken::LCurly);
        ptr.expect(ContentToken::Whitespace);
    }

    ptr.expect(ContentToken::Word);

    if group {
        ptr.expect(ContentToken::Whitespace);
        ptr.expect(ContentToken::RCurly);
    }

    ptr.builder.finish_node();
    ptr
}

fn command(mut ptr: TokenPtr<ContentToken>) -> TokenPtr<ContentToken> {
    ptr.builder.start_node(COMMAND.into());
    ptr.bump();
    ptr.builder.finish_node();
    ptr
}

struct TokenPtr<'a, T: Logos<'a>> {
    builder: GreenNodeBuilder<'static>,
    lexer: logos::Lexer<'a, T>,
    token: Option<(T, &'a str)>,
}

impl<'a, T> TokenPtr<'a, T>
where
    T: Logos<'a, Source = str> + Eq + Copy + Into<SyntaxKind>,
    T::Extras: Default,
{
    pub fn morph<U>(mut self) -> TokenPtr<'a, U>
    where
        U: Logos<'a, Source = str> + Eq + Copy + Into<SyntaxKind>,
        U::Extras: Default,
    {
        self.peek();
        let start = self.lexer.span().start;
        let input = &self.lexer.source()[start..];
        TokenPtr {
            builder: self.builder,
            lexer: U::lexer(input),
            token: None,
        }
    }

    #[must_use]
    pub fn at(&mut self, kind: T) -> bool {
        self.peek().map_or(false, |(k, _)| k == kind)
    }

    #[must_use]
    pub fn current(&mut self) -> Option<T> {
        self.peek().map(|(k, _)| k)
    }

    pub fn bump(&mut self) {
        let (kind, text) = self.peek().unwrap();
        self.token = None;
        self.builder
            .token(rowan::SyntaxKind::from(kind.into()), text);
    }

    pub fn expect(&mut self, kind: T) {
        if self.at(kind) {
            self.bump();
        }
    }

    fn peek(&mut self) -> Option<(T, &'a str)> {
        if self.token.is_none() {
            let kind = self.lexer.next()?;
            let text = self.lexer.slice();
            self.token = Some((kind, text));
        }

        self.token
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, Logos)]
enum RootToken {
    #[token(r"@preamble", ignore(ascii_case))]
    Preamble,

    #[token(r"@string", ignore(ascii_case))]
    String,

    #[token(r"@comment", ignore(ascii_case))]
    Comment,

    #[regex(r"@[a-zA-Z]*")]
    Entry,

    #[regex(r"[^@]+")]
    #[error]
    Junk,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, Logos)]
enum BodyToken {
    #[regex(r"\s+")]
    Whitespace,

    #[token("{")]
    #[token("(")]
    LDelim,

    #[token("}")]
    #[token(")")]
    RDelim,

    #[token(",")]
    Comma,

    #[token("=")]
    Eq,

    #[regex(r"[^\s\(\)\{\}@,=]+")]
    Name,

    #[error]
    Error,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, Logos)]
enum ValueToken {
    #[regex(r"\s+")]
    Whitespace,

    #[token("#")]
    Pound,

    #[token(",")]
    Comma,

    #[token("{")]
    LCurly,

    #[token("}")]
    RCurly,

    #[token("\"")]
    Quote,

    #[regex(r"\d+", priority = 2)]
    Integer,

    #[regex(r#"[^\s"\{\},]+"#)]
    #[error]
    Name,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, Logos)]
enum ContentToken {
    #[regex(r"\s+")]
    Whitespace,

    #[token(",")]
    Comma,

    #[token("{")]
    LCurly,

    #[token("}")]
    RCurly,

    #[token("\"")]
    Quote,

    #[regex(r"\d+", priority = 2)]
    Integer,

    #[token(r#"~"#)]
    Nbsp,

    #[token(r#"\`"#)]
    #[token(r#"\'"#)]
    #[token(r#"\^"#)]
    #[token(r#"\""#)]
    #[token(r#"\H"#)]
    #[token(r#"\~"#)]
    #[token(r#"\c"#)]
    #[token(r#"\k"#)]
    #[token(r#"\="#)]
    #[token(r#"\b"#)]
    #[token(r#"\."#)]
    #[token(r#"\d"#)]
    #[token(r#"\r"#)]
    #[token(r#"\u"#)]
    #[token(r#"\v"#)]
    #[token(r#"\t"#)]
    AccentName,

    #[regex(r"\\([^\r\n]|[@a-zA-Z:_]+\*?)?")]
    CommandName,

    #[regex(r#"[^\s"\{\}\\~,]+"#)]
    #[error]
    Word,
}

impl From<RootToken> for SyntaxKind {
    fn from(token: RootToken) -> Self {
        match token {
            RootToken::Preamble | RootToken::String | RootToken::Comment | RootToken::Entry => TYPE,
            RootToken::Junk => JUNK,
        }
    }
}

impl From<BodyToken> for SyntaxKind {
    fn from(token: BodyToken) -> Self {
        match token {
            BodyToken::Whitespace => WHITESPACE,
            BodyToken::LDelim => L_DELIM,
            BodyToken::RDelim => R_DELIM,
            BodyToken::Comma => COMMA,
            BodyToken::Eq => EQ,
            BodyToken::Name => NAME,
            BodyToken::Error => unreachable!(),
        }
    }
}

impl From<ValueToken> for SyntaxKind {
    fn from(token: ValueToken) -> Self {
        match token {
            ValueToken::Whitespace => WHITESPACE,
            ValueToken::Pound => POUND,
            ValueToken::Comma => COMMA,
            ValueToken::LCurly => L_CURLY,
            ValueToken::RCurly => R_CURLY,
            ValueToken::Quote => QUOTE,
            ValueToken::Integer => INTEGER,
            ValueToken::Name => NAME,
        }
    }
}

impl From<ContentToken> for SyntaxKind {
    fn from(token: ContentToken) -> Self {
        match token {
            ContentToken::Whitespace => WHITESPACE,
            ContentToken::Comma => COMMA,
            ContentToken::LCurly => L_CURLY,
            ContentToken::RCurly => R_CURLY,
            ContentToken::Quote => QUOTE,
            ContentToken::Integer => INTEGER,
            ContentToken::Nbsp => NBSP,
            ContentToken::AccentName => ACCENT_NAME,
            ContentToken::CommandName => COMMAND_NAME,
            ContentToken::Word => WORD,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::syntax::bibtex;

    use super::parse_bibtex;

    #[test]
    fn test_parse() {
        insta::glob!("test_data/bibtex/{,**/}*.txt", |path| {
            let text = std::fs::read_to_string(path).unwrap().replace("\r\n", "\n");
            let root = bibtex::SyntaxNode::new_root(parse_bibtex(&text));
            insta::assert_debug_snapshot!(root);
        });
    }
}
