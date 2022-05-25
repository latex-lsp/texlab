use logos::{Lexer, Logos};
use rowan::{GreenNode, GreenNodeBuilder};

use super::{
    lexer::*,
    SyntaxKind::{self, *},
};

pub fn parse(input: &str) -> GreenNode {
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
    lexer: Lexer<'a, T>,
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
