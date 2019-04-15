use crate::syntax::text::{CharStream, Span};
use lsp_types::{Position, Range};
use std::rc::Rc;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum BibtexTokenKind {
    PreambleType,
    StringType,
    EntryType,
    Word,
    Command,
    Assign,
    Comma,
    Concat,
    Quote,
    BeginBrace,
    EndBrace,
    BeginParen,
    EndParen,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BibtexToken {
    pub span: Span,
    pub kind: BibtexTokenKind,
}

impl BibtexToken {
    pub fn new(span: Span, kind: BibtexTokenKind) -> Self {
        BibtexToken { span, kind }
    }

    pub fn range(&self) -> Range {
        self.span.range
    }

    pub fn start(&self) -> Position {
        self.span.start()
    }

    pub fn end(&self) -> Position {
        self.span.end()
    }

    pub fn text(&self) -> &str {
        &self.span.text
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BibtexRoot {
    pub children: Vec<BibtexDeclaration>,
}

impl BibtexRoot {
    pub fn new(children: Vec<BibtexDeclaration>) -> Self {
        BibtexRoot { children }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum BibtexDeclaration {
    Comment(Rc<BibtexComment>),
    Preamble(Rc<BibtexPreamble>),
    String(Rc<BibtexString>),
    Entry(Rc<BibtexEntry>),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BibtexComment {
    pub range: Range,
    pub token: BibtexToken,
}

impl BibtexComment {
    pub fn new(token: BibtexToken) -> Self {
        BibtexComment {
            range: token.range(),
            token,
        }
    }
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BibtexPreamble {
    pub range: Range,
    pub kind: BibtexToken,
    pub left: Option<BibtexToken>,
    pub content: Option<BibtexContent>,
    pub right: Option<BibtexToken>,
}

impl BibtexPreamble {
    pub fn new(
        kind: BibtexToken,
        left: Option<BibtexToken>,
        content: Option<BibtexContent>,
        right: Option<BibtexToken>,
    ) -> Self {
        let end = if let Some(ref right) = right {
            right.end()
        } else if let Some(ref content) = content {
            content.end()
        } else if let Some(ref left) = left {
            left.end()
        } else {
            kind.end()
        };
        BibtexPreamble {
            range: Range::new(kind.start(), end),
            kind,
            left,
            content,
            right,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BibtexString {
    pub range: Range,
    pub kind: BibtexToken,
    pub left: Option<BibtexToken>,
    pub name: Option<BibtexToken>,
    pub assign: Option<BibtexToken>,
    pub value: Option<BibtexContent>,
    pub right: Option<BibtexToken>,
}

impl BibtexString {
    pub fn new(
        kind: BibtexToken,
        left: Option<BibtexToken>,
        name: Option<BibtexToken>,
        assign: Option<BibtexToken>,
        value: Option<BibtexContent>,
        right: Option<BibtexToken>,
    ) -> Self {
        let end = if let Some(ref right) = right {
            right.end()
        } else if let Some(ref value) = value {
            value.end()
        } else if let Some(ref assign) = assign {
            assign.end()
        } else if let Some(ref name) = name {
            name.end()
        } else if let Some(ref left) = left {
            left.end()
        } else {
            kind.end()
        };

        BibtexString {
            range: Range::new(kind.start(), end),
            kind,
            left,
            name,
            assign,
            value,
            right,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BibtexEntry {
    pub range: Range,
    pub kind: BibtexToken,
    pub left: Option<BibtexToken>,
    pub key: Option<BibtexToken>,
    pub comma: Option<BibtexToken>,
    pub fields: Vec<BibtexField>,
    pub right: Option<BibtexToken>,
}

impl BibtexEntry {
    pub fn new(
        kind: BibtexToken,
        left: Option<BibtexToken>,
        key: Option<BibtexToken>,
        comma: Option<BibtexToken>,
        fields: Vec<BibtexField>,
        right: Option<BibtexToken>,
    ) -> Self {
        let end = if let Some(ref right) = right {
            right.end()
        } else if !fields.is_empty() {
            fields[fields.len() - 1].range.end
        } else if let Some(ref comma) = comma {
            comma.end()
        } else if let Some(ref key) = key {
            key.end()
        } else if let Some(ref left) = left {
            left.end()
        } else {
            kind.end()
        };

        BibtexEntry {
            range: Range::new(kind.start(), end),
            kind,
            left,
            key,
            comma,
            fields,
            right,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BibtexField {
    pub range: Range,
    pub name: BibtexToken,
    pub assign: Option<BibtexToken>,
    pub content: Option<BibtexContent>,
    pub comma: Option<BibtexToken>,
}

impl BibtexField {
    pub fn new(
        name: BibtexToken,
        assign: Option<BibtexToken>,
        content: Option<BibtexContent>,
        comma: Option<BibtexToken>,
    ) -> Self {
        let end = if let Some(ref comma) = comma {
            comma.end()
        } else if let Some(ref content) = content {
            content.end()
        } else if let Some(ref assign) = assign {
            assign.end()
        } else {
            name.end()
        };

        BibtexField {
            range: Range::new(name.start(), end),
            name,
            assign,
            content,
            comma,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum BibtexContent {
    Word(Rc<BibtexWord>),
    Command(Rc<BibtexCommand>),
    QuotedContent(Rc<BibtexQuotedContent>),
    BracedContent(Rc<BibtexBracedContent>),
    Concat(Rc<BibtexConcat>),
}

impl BibtexContent {
    pub fn range(&self) -> Range {
        match self {
            BibtexContent::Word(word) => word.range,
            BibtexContent::Command(command) => command.range,
            BibtexContent::QuotedContent(content) => content.range,
            BibtexContent::BracedContent(content) => content.range,
            BibtexContent::Concat(concat) => concat.range,
        }
    }

    pub fn start(&self) -> Position {
        self.range().start
    }

    pub fn end(&self) -> Position {
        self.range().end
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BibtexWord {
    pub range: Range,
    pub token: BibtexToken,
}

impl BibtexWord {
    pub fn new(token: BibtexToken) -> Self {
        BibtexWord {
            range: token.range(),
            token,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BibtexCommand {
    pub range: Range,
    pub token: BibtexToken,
}

impl BibtexCommand {
    pub fn new(token: BibtexToken) -> Self {
        BibtexCommand {
            range: token.range(),
            token,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BibtexQuotedContent {
    pub range: Range,
    pub left: BibtexToken,
    pub children: Vec<BibtexContent>,
    pub right: Option<BibtexToken>,
}

impl BibtexQuotedContent {
    pub fn new(
        left: BibtexToken,
        children: Vec<BibtexContent>,
        right: Option<BibtexToken>,
    ) -> Self {
        let end = if let Some(ref right) = right {
            right.end()
        } else if !children.is_empty() {
            children[children.len() - 1].end()
        } else {
            left.end()
        };

        BibtexQuotedContent {
            range: Range::new(left.start(), end),
            left,
            children,
            right,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BibtexBracedContent {
    pub range: Range,
    pub left: BibtexToken,
    pub children: Vec<BibtexContent>,
    pub right: Option<BibtexToken>,
}

impl BibtexBracedContent {
    pub fn new(
        left: BibtexToken,
        children: Vec<BibtexContent>,
        right: Option<BibtexToken>,
    ) -> Self {
        let end = if let Some(ref right) = right {
            right.end()
        } else if !children.is_empty() {
            children[children.len() - 1].end()
        } else {
            left.end()
        };

        BibtexBracedContent {
            range: Range::new(left.start(), end),
            left,
            children,
            right,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BibtexConcat {
    pub range: Range,
    pub left: BibtexContent,
    pub operator: BibtexToken,
    pub right: Option<BibtexContent>,
}

impl BibtexConcat {
    pub fn new(left: BibtexContent, operator: BibtexToken, right: Option<BibtexContent>) -> Self {
        let end = if let Some(ref right) = right {
            right.end()
        } else {
            operator.end()
        };

        BibtexConcat {
            range: Range::new(left.start(), end),
            left,
            operator,
            right,
        }
    }
}
