use crate::text::{Span, SyntaxNode};
use lsp_types::Range;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum BibtexTokenKind {
    PreambleKind,
    StringKind,
    EntryKind,
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

    pub fn text(&self) -> &str {
        &self.span.text
    }
}

impl SyntaxNode for BibtexToken {
    fn range(&self) -> Range {
        self.span.range
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

impl SyntaxNode for BibtexRoot {
    fn range(&self) -> Range {
        if self.children.is_empty() {
            Range::new_simple(0, 0, 0, 0)
        } else {
            Range::new(
                self.children[0].start(),
                self.children[self.children.len() - 1].end(),
            )
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum BibtexDeclaration {
    Comment(Box<BibtexComment>),
    Preamble(Box<BibtexPreamble>),
    String(Box<BibtexString>),
    Entry(Box<BibtexEntry>),
}

impl BibtexDeclaration {
    pub fn accept<'a>(&'a self, visitor: &mut BibtexVisitor<'a>) {
        match self {
            BibtexDeclaration::Comment(comment) => visitor.visit_comment(comment),
            BibtexDeclaration::Preamble(preamble) => visitor.visit_preamble(preamble),
            BibtexDeclaration::String(string) => visitor.visit_string(string),
            BibtexDeclaration::Entry(entry) => visitor.visit_entry(entry),
        }
    }
}

impl SyntaxNode for BibtexDeclaration {
    fn range(&self) -> Range {
        match self {
            BibtexDeclaration::Comment(comment) => comment.range,
            BibtexDeclaration::Preamble(preamble) => preamble.range,
            BibtexDeclaration::String(string) => string.range,
            BibtexDeclaration::Entry(entry) => entry.range,
        }
    }
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

impl SyntaxNode for BibtexComment {
    fn range(&self) -> Range {
        self.range
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BibtexPreamble {
    pub range: Range,
    pub ty: BibtexToken,
    pub left: Option<BibtexToken>,
    pub content: Option<BibtexContent>,
    pub right: Option<BibtexToken>,
}

impl BibtexPreamble {
    pub fn new(
        ty: BibtexToken,
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
            ty.end()
        };
        BibtexPreamble {
            range: Range::new(ty.start(), end),
            ty,
            left,
            content,
            right,
        }
    }
}

impl SyntaxNode for BibtexPreamble {
    fn range(&self) -> Range {
        self.range
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BibtexString {
    pub range: Range,
    pub ty: BibtexToken,
    pub left: Option<BibtexToken>,
    pub name: Option<BibtexToken>,
    pub assign: Option<BibtexToken>,
    pub value: Option<BibtexContent>,
    pub right: Option<BibtexToken>,
}

impl BibtexString {
    pub fn new(
        ty: BibtexToken,
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
            ty.end()
        };

        BibtexString {
            range: Range::new(ty.start(), end),
            ty,
            left,
            name,
            assign,
            value,
            right,
        }
    }
}

impl SyntaxNode for BibtexString {
    fn range(&self) -> Range {
        self.range
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BibtexEntry {
    pub range: Range,
    pub ty: BibtexToken,
    pub left: Option<BibtexToken>,
    pub key: Option<BibtexToken>,
    pub comma: Option<BibtexToken>,
    pub fields: Vec<BibtexField>,
    pub right: Option<BibtexToken>,
}

impl BibtexEntry {
    pub fn new(
        ty: BibtexToken,
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
            ty.end()
        };

        BibtexEntry {
            range: Range::new(ty.start(), end),
            ty,
            left,
            key,
            comma,
            fields,
            right,
        }
    }

    pub fn is_comment(&self) -> bool {
        self.ty.text().to_lowercase() == "@comment"
    }
}

impl SyntaxNode for BibtexEntry {
    fn range(&self) -> Range {
        self.range
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

impl SyntaxNode for BibtexField {
    fn range(&self) -> Range {
        self.range
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum BibtexContent {
    Word(BibtexWord),
    Command(BibtexCommand),
    QuotedContent(BibtexQuotedContent),
    BracedContent(BibtexBracedContent),
    Concat(Box<BibtexConcat>),
}

impl BibtexContent {
    pub fn accept<'a>(&'a self, visitor: &mut BibtexVisitor<'a>) {
        match self {
            BibtexContent::Word(word) => visitor.visit_word(word),
            BibtexContent::Command(command) => visitor.visit_command(command),
            BibtexContent::QuotedContent(content) => visitor.visit_quoted_content(content),
            BibtexContent::BracedContent(content) => visitor.visit_braced_content(content),
            BibtexContent::Concat(concat) => visitor.visit_concat(concat),
        }
    }
}

impl SyntaxNode for BibtexContent {
    fn range(&self) -> Range {
        match self {
            BibtexContent::Word(word) => word.range(),
            BibtexContent::Command(command) => command.range(),
            BibtexContent::QuotedContent(content) => content.range(),
            BibtexContent::BracedContent(content) => content.range(),
            BibtexContent::Concat(concat) => concat.range(),
        }
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

impl SyntaxNode for BibtexWord {
    fn range(&self) -> Range {
        self.range
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

impl SyntaxNode for BibtexCommand {
    fn range(&self) -> Range {
        self.range
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

impl SyntaxNode for BibtexQuotedContent {
    fn range(&self) -> Range {
        self.range
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

impl SyntaxNode for BibtexBracedContent {
    fn range(&self) -> Range {
        self.range
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

impl SyntaxNode for BibtexConcat {
    fn range(&self) -> Range {
        self.range
    }
}

pub trait BibtexVisitor<'a> {
    fn visit_root(&mut self, root: &'a BibtexRoot);

    fn visit_comment(&mut self, comment: &'a BibtexComment);

    fn visit_preamble(&mut self, preamble: &'a BibtexPreamble);

    fn visit_string(&mut self, string: &'a BibtexString);

    fn visit_entry(&mut self, entry: &'a BibtexEntry);

    fn visit_field(&mut self, field: &'a BibtexField);

    fn visit_word(&mut self, word: &'a BibtexWord);

    fn visit_command(&mut self, command: &'a BibtexCommand);

    fn visit_quoted_content(&mut self, content: &'a BibtexQuotedContent);

    fn visit_braced_content(&mut self, content: &'a BibtexBracedContent);

    fn visit_concat(&mut self, concat: &'a BibtexConcat);
}

pub struct BibtexWalker;

impl BibtexWalker {
    pub fn walk_root<'a>(visitor: &mut BibtexVisitor<'a>, root: &'a BibtexRoot) {
        for declaration in &root.children {
            declaration.accept(visitor);
        }
    }

    pub fn walk_preamble<'a>(visitor: &mut BibtexVisitor<'a>, preamble: &'a BibtexPreamble) {
        if let Some(ref content) = preamble.content {
            content.accept(visitor);
        }
    }

    pub fn walk_string<'a>(visitor: &mut BibtexVisitor<'a>, string: &'a BibtexString) {
        if let Some(ref value) = string.value {
            value.accept(visitor);
        }
    }

    pub fn walk_entry<'a>(visitor: &mut BibtexVisitor<'a>, entry: &'a BibtexEntry) {
        for field in &entry.fields {
            visitor.visit_field(field);
        }
    }

    pub fn walk_field<'a>(visitor: &mut BibtexVisitor<'a>, field: &'a BibtexField) {
        if let Some(ref content) = field.content {
            content.accept(visitor);
        }
    }

    pub fn walk_quoted_content<'a>(
        visitor: &mut BibtexVisitor<'a>,
        content: &'a BibtexQuotedContent,
    ) {
        for child in &content.children {
            child.accept(visitor);
        }
    }

    pub fn walk_braced_content<'a>(
        visitor: &mut BibtexVisitor<'a>,
        content: &'a BibtexBracedContent,
    ) {
        for child in &content.children {
            child.accept(visitor);
        }
    }

    pub fn walk_concat<'a>(visitor: &mut BibtexVisitor<'a>, concat: &'a BibtexConcat) {
        concat.left.accept(visitor);
        if let Some(ref right) = concat.right {
            right.accept(visitor);
        }
    }
}
