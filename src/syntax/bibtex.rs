use crate::range;
use crate::syntax::text::{CharStream, Span};
use lsp_types::{Position, Range};
use std::iter::Peekable;
use std::rc::Rc;

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

pub trait BibtexVisitor<T> {
    fn visit_comment(&mut self, comment: Rc<BibtexComment>) -> T;

    fn visit_preamble(&mut self, preamble: Rc<BibtexPreamble>) -> T;

    fn visit_string(&mut self, string: Rc<BibtexString>) -> T;

    fn visit_entry(&mut self, entry: Rc<BibtexEntry>) -> T;

    fn visit_field(&mut self, field: Rc<BibtexField>) -> T;

    fn visit_word(&mut self, word: Rc<BibtexWord>) -> T;

    fn visit_command(&mut self, command: Rc<BibtexCommand>) -> T;

    fn visit_quoted_content(&mut self, content: Rc<BibtexQuotedContent>) -> T;

    fn visit_braced_content(&mut self, content: Rc<BibtexBracedContent>) -> T;

    fn visit_concat(&mut self, concat: Rc<BibtexConcat>) -> T;
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum BibtexDeclaration {
    Comment(Rc<BibtexComment>),
    Preamble(Rc<BibtexPreamble>),
    String(Rc<BibtexString>),
    Entry(Rc<BibtexEntry>),
}

impl BibtexDeclaration {
    pub fn range(&self) -> Range {
        match self {
            BibtexDeclaration::Comment(comment) => comment.range,
            BibtexDeclaration::Preamble(preamble) => preamble.range,
            BibtexDeclaration::String(string) => string.range,
            BibtexDeclaration::Entry(entry) => entry.range,
        }
    }

    pub fn start(&self) -> Position {
        self.range().start
    }

    pub fn end(&self) -> Position {
        self.range().end
    }

    pub fn accept<T>(&self, visitor: &mut BibtexVisitor<T>) -> T {
        match self {
            BibtexDeclaration::Comment(comment) => visitor.visit_comment(comment.clone()),
            BibtexDeclaration::Preamble(preamble) => visitor.visit_preamble(preamble.clone()),
            BibtexDeclaration::String(string) => visitor.visit_string(string.clone()),
            BibtexDeclaration::Entry(entry) => visitor.visit_entry(entry.clone()),
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
    pub fields: Vec<Rc<BibtexField>>,
    pub right: Option<BibtexToken>,
}

impl BibtexEntry {
    pub fn new(
        kind: BibtexToken,
        left: Option<BibtexToken>,
        key: Option<BibtexToken>,
        comma: Option<BibtexToken>,
        fields: Vec<Rc<BibtexField>>,
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

    pub fn accept<T>(&self, visitor: &mut BibtexVisitor<T>) -> T {
        match self {
            BibtexContent::Word(word) => visitor.visit_word(word.clone()),
            BibtexContent::Command(command) => visitor.visit_command(command.clone()),
            BibtexContent::QuotedContent(content) => visitor.visit_quoted_content(content.clone()),
            BibtexContent::BracedContent(content) => visitor.visit_braced_content(content.clone()),
            BibtexContent::Concat(concat) => visitor.visit_concat(concat.clone()),
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

struct BibtexLexer<'a> {
    stream: CharStream<'a>,
}

impl<'a> From<CharStream<'a>> for BibtexLexer<'a> {
    fn from(stream: CharStream<'a>) -> Self {
        BibtexLexer { stream }
    }
}

impl<'a> From<&'a str> for BibtexLexer<'a> {
    fn from(text: &'a str) -> Self {
        let stream = CharStream::new(text);
        BibtexLexer::from(stream)
    }
}

impl<'a> BibtexLexer<'a> {
    fn kind(&mut self) -> BibtexToken {
        fn is_type_char(c: char) -> bool {
            c >= 'a' && c <= 'z' || c >= 'A' && c <= 'Z'
        }

        self.stream.start_span();
        self.stream.next().unwrap();
        while self.stream.satifies(|c| is_type_char(*c)) {
            self.stream.next();
        }
        let span = self.stream.end_span();
        let kind = match span.text.as_ref() {
            "@preamble" => BibtexTokenKind::PreambleKind,
            "@string" => BibtexTokenKind::StringKind,
            _ => BibtexTokenKind::EntryKind,
        };
        BibtexToken::new(span, kind)
    }

    fn single_character(&mut self, kind: BibtexTokenKind) -> BibtexToken {
        self.stream.start_span();
        self.stream.next();
        let span = self.stream.end_span();
        BibtexToken::new(span, kind)
    }

    fn command(&mut self) -> BibtexToken {
        let span = self.stream.command();
        BibtexToken::new(span, BibtexTokenKind::Command)
    }

    fn word(&mut self) -> BibtexToken {
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
        BibtexToken::new(span, BibtexTokenKind::Word)
    }
}

impl<'a> Iterator for BibtexLexer<'a> {
    type Item = BibtexToken;

    fn next(&mut self) -> Option<BibtexToken> {
        loop {
            match self.stream.peek() {
                Some('@') => return Some(self.kind()),
                Some('=') => return Some(self.single_character(BibtexTokenKind::Assign)),
                Some(',') => return Some(self.single_character(BibtexTokenKind::Comma)),
                Some('#') => return Some(self.single_character(BibtexTokenKind::Concat)),
                Some('"') => return Some(self.single_character(BibtexTokenKind::Quote)),
                Some('{') => return Some(self.single_character(BibtexTokenKind::BeginBrace)),
                Some('}') => return Some(self.single_character(BibtexTokenKind::EndBrace)),
                Some('(') => return Some(self.single_character(BibtexTokenKind::BeginParen)),
                Some(')') => return Some(self.single_character(BibtexTokenKind::EndParen)),
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

struct BibtexParser<I: Iterator<Item = BibtexToken>> {
    tokens: Peekable<I>,
}

impl<I: Iterator<Item = BibtexToken>> BibtexParser<I> {
    fn new(tokens: I) -> Self {
        BibtexParser {
            tokens: tokens.peekable(),
        }
    }

    fn root(&mut self) -> BibtexRoot {
        let mut children = Vec::new();
        while let Some(ref token) = self.tokens.peek() {
            match token.kind {
                BibtexTokenKind::PreambleKind => {
                    children.push(BibtexDeclaration::Preamble(Rc::new(self.preamble())));
                }
                BibtexTokenKind::StringKind => {
                    children.push(BibtexDeclaration::String(Rc::new(self.string())));
                }
                BibtexTokenKind::EntryKind => {
                    children.push(BibtexDeclaration::Entry(Rc::new(self.entry())));
                }
                _ => {
                    let token = self.tokens.next().unwrap();
                    let comment = BibtexComment::new(token);
                    children.push(BibtexDeclaration::Comment(Rc::new(comment)));
                }
            }
        }
        BibtexRoot::new(children)
    }

    fn preamble(&mut self) -> BibtexPreamble {
        let kind = self.tokens.next().unwrap();

        let left = self.expect2(BibtexTokenKind::BeginBrace, BibtexTokenKind::BeginParen);
        if left.is_none() {
            return BibtexPreamble::new(kind, None, None, None);
        }

        if !self.can_match_content() {
            return BibtexPreamble::new(kind, left, None, None);
        }
        let content = self.content();

        let right = self.expect2(BibtexTokenKind::EndBrace, BibtexTokenKind::EndParen);
        BibtexPreamble::new(kind, left, Some(content), right)
    }

    fn string(&mut self) -> BibtexString {
        let kind = self.tokens.next().unwrap();

        let left = self.expect2(BibtexTokenKind::BeginBrace, BibtexTokenKind::BeginParen);
        if left.is_none() {
            return BibtexString::new(kind, None, None, None, None, None);
        }

        let name = self.expect1(BibtexTokenKind::Word);
        if name.is_none() {
            return BibtexString::new(kind, left, None, None, None, None);
        }

        let assign = self.expect1(BibtexTokenKind::Assign);
        if assign.is_none() {
            return BibtexString::new(kind, left, name, None, None, None);
        }

        if !self.can_match_content() {
            return BibtexString::new(kind, left, name, assign, None, None);
        }
        let value = self.content();

        let right = self.expect2(BibtexTokenKind::EndBrace, BibtexTokenKind::EndParen);
        BibtexString::new(kind, left, name, assign, Some(value), right)
    }

    fn entry(&mut self) -> BibtexEntry {
        let kind = self.tokens.next().unwrap();

        let left = self.expect2(BibtexTokenKind::BeginBrace, BibtexTokenKind::BeginParen);
        if left.is_none() {
            return BibtexEntry::new(kind, None, None, None, Vec::new(), None);
        }

        let name = self.expect1(BibtexTokenKind::Word);
        if name.is_none() {
            return BibtexEntry::new(kind, left, None, None, Vec::new(), None);
        }

        let comma = self.expect1(BibtexTokenKind::Comma);
        if comma.is_none() {
            return BibtexEntry::new(kind, left, name, None, Vec::new(), None);
        }

        let mut fields = Vec::new();
        while self.next_of_kind(BibtexTokenKind::Word) {
            fields.push(Rc::new(self.field()));
        }

        let right = self.expect2(BibtexTokenKind::EndBrace, BibtexTokenKind::EndParen);
        BibtexEntry::new(kind, left, name, comma, fields, right)
    }

    fn field(&mut self) -> BibtexField {
        let name = self.tokens.next().unwrap();

        let assign = self.expect1(BibtexTokenKind::Assign);
        if assign.is_none() {
            return BibtexField::new(name, None, None, None);
        }

        if !self.can_match_content() {
            return BibtexField::new(name, assign, None, None);
        }
        let content = self.content();

        let comma = self.expect1(BibtexTokenKind::Comma);
        BibtexField::new(name, assign, Some(content), comma)
    }

    fn content(&mut self) -> BibtexContent {
        let token = self.tokens.next().unwrap();
        let left = match token.kind {
            BibtexTokenKind::PreambleKind
            | BibtexTokenKind::StringKind
            | BibtexTokenKind::EntryKind
            | BibtexTokenKind::Word
            | BibtexTokenKind::Assign
            | BibtexTokenKind::Comma
            | BibtexTokenKind::BeginParen
            | BibtexTokenKind::EndParen => BibtexContent::Word(Rc::new(BibtexWord::new(token))),
            BibtexTokenKind::Command => BibtexContent::Command(Rc::new(BibtexCommand::new(token))),
            BibtexTokenKind::Quote => {
                let mut children = Vec::new();
                while self.can_match_content() {
                    if self.next_of_kind(BibtexTokenKind::Quote) {
                        break;
                    }
                    children.push(self.content());
                }
                let right = self.expect1(BibtexTokenKind::Quote);
                BibtexContent::QuotedContent(Rc::new(BibtexQuotedContent::new(
                    token, children, right,
                )))
            }
            BibtexTokenKind::BeginBrace => {
                let mut children = Vec::new();
                while self.can_match_content() {
                    children.push(self.content());
                }
                let right = self.expect1(BibtexTokenKind::EndBrace);
                BibtexContent::BracedContent(Rc::new(BibtexBracedContent::new(
                    token, children, right,
                )))
            }
            _ => unreachable!(),
        };
        if let Some(operator) = self.expect1(BibtexTokenKind::Concat) {
            let right = if self.can_match_content() {
                Some(self.content())
            } else {
                None
            };
            BibtexContent::Concat(Rc::new(BibtexConcat::new(left, operator, right)))
        } else {
            left
        }
    }

    fn can_match_content(&mut self) -> bool {
        if let Some(ref token) = self.tokens.peek() {
            match token.kind {
                BibtexTokenKind::PreambleKind
                | BibtexTokenKind::StringKind
                | BibtexTokenKind::EntryKind
                | BibtexTokenKind::Word
                | BibtexTokenKind::Command
                | BibtexTokenKind::Assign
                | BibtexTokenKind::Comma
                | BibtexTokenKind::Quote
                | BibtexTokenKind::BeginBrace
                | BibtexTokenKind::BeginParen
                | BibtexTokenKind::EndParen => true,
                BibtexTokenKind::Concat | BibtexTokenKind::EndBrace => false,
            }
        } else {
            false
        }
    }

    fn expect1(&mut self, kind: BibtexTokenKind) -> Option<BibtexToken> {
        if let Some(ref token) = self.tokens.peek() {
            if token.kind == kind {
                return self.tokens.next();
            }
        }
        None
    }

    fn expect2(&mut self, kind1: BibtexTokenKind, kind2: BibtexTokenKind) -> Option<BibtexToken> {
        if let Some(ref token) = self.tokens.peek() {
            if token.kind == kind1 || token.kind == kind2 {
                return self.tokens.next();
            }
        }
        None
    }

    fn next_of_kind(&mut self, kind: BibtexTokenKind) -> bool {
        if let Some(token) = self.tokens.peek() {
            token.kind == kind
        } else {
            false
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum BibtexNodeKind {
    Comment,
    Preamble,
    String,
    Entry,
    Field,
    Word,
    Command,
    QuotedContent,
    BracedContent,
    Concat,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum BibtexNode {
    Declaration(BibtexDeclaration),
    Field(Rc<BibtexField>),
    Content(BibtexContent),
}

impl BibtexNode {
    fn range(&self) -> Range {
        match self {
            BibtexNode::Declaration(declaration) => declaration.range(),
            BibtexNode::Field(field) => field.range,
            BibtexNode::Content(content) => content.range(),
        }
    }

    fn kind(&self) -> BibtexNodeKind {
        match self {
            BibtexNode::Declaration(declaration) => match declaration {
                BibtexDeclaration::Comment(_) => BibtexNodeKind::Comment,
                BibtexDeclaration::Preamble(_) => BibtexNodeKind::Preamble,
                BibtexDeclaration::String(_) => BibtexNodeKind::String,
                BibtexDeclaration::Entry(_) => BibtexNodeKind::Entry,
            },
            BibtexNode::Field(_) => BibtexNodeKind::Field,
            BibtexNode::Content(content) => match content {
                BibtexContent::Word(_) => BibtexNodeKind::Word,
                BibtexContent::Command(_) => BibtexNodeKind::Command,
                BibtexContent::QuotedContent(_) => BibtexNodeKind::QuotedContent,
                BibtexContent::BracedContent(_) => BibtexNodeKind::BracedContent,
                BibtexContent::Concat(_) => BibtexNodeKind::Concat,
            },
        }
    }
}

struct BibtexFinder {
    pub position: Option<Position>,
    pub results: Vec<BibtexNode>,
}

impl BibtexFinder {
    pub fn new(position: Option<Position>) -> Self {
        BibtexFinder {
            position,
            results: Vec::new(),
        }
    }

    fn check_range(&self, node: &BibtexNode) -> bool {
        if let Some(position) = self.position {
            range::contains(node.range(), position)
        } else {
            true
        }
    }
}

impl BibtexVisitor<()> for BibtexFinder {
    fn visit_comment(&mut self, comment: Rc<BibtexComment>) {
        let node = BibtexNode::Declaration(BibtexDeclaration::Comment(Rc::clone(&comment)));
        if self.check_range(&node) {
            self.results.push(node);
        }
    }

    fn visit_preamble(&mut self, preamble: Rc<BibtexPreamble>) {
        let node = BibtexNode::Declaration(BibtexDeclaration::Preamble(Rc::clone(&preamble)));
        if self.check_range(&node) {
            self.results.push(node);
            if let Some(ref content) = preamble.content {
                content.accept(self);
            }
        }
    }

    fn visit_string(&mut self, string: Rc<BibtexString>) {
        let node = BibtexNode::Declaration(BibtexDeclaration::String(Rc::clone(&string)));
        if self.check_range(&node) {
            self.results.push(node);
            if let Some(ref content) = string.value {
                content.accept(self);
            }
        }
    }

    fn visit_entry(&mut self, entry: Rc<BibtexEntry>) {
        let node = BibtexNode::Declaration(BibtexDeclaration::Entry(Rc::clone(&entry)));
        if self.check_range(&node) {
            self.results.push(node);
            for field in &entry.fields {
                self.visit_field(Rc::clone(&field));
            }
        }
    }

    fn visit_field(&mut self, field: Rc<BibtexField>) {
        let node = BibtexNode::Field(Rc::clone(&field));
        if self.check_range(&node) {
            self.results.push(node);
            if let Some(ref content) = field.content {
                content.accept(self);
            }
        }
    }

    fn visit_word(&mut self, word: Rc<BibtexWord>) {
        let node = BibtexNode::Content(BibtexContent::Word(Rc::clone(&word)));
        if self.check_range(&node) {
            self.results.push(node);
        }
    }

    fn visit_command(&mut self, command: Rc<BibtexCommand>) {
        let node = BibtexNode::Content(BibtexContent::Command(Rc::clone(&command)));
        if self.check_range(&node) {
            self.results.push(node);
        }
    }

    fn visit_quoted_content(&mut self, content: Rc<BibtexQuotedContent>) {
        let node = BibtexNode::Content(BibtexContent::QuotedContent(Rc::clone(&content)));
        if self.check_range(&node) {
            self.results.push(node);
            for child in &content.children {
                child.accept(self);
            }
        }
    }

    fn visit_braced_content(&mut self, content: Rc<BibtexBracedContent>) {
        let node = BibtexNode::Content(BibtexContent::BracedContent(Rc::clone(&content)));
        if self.check_range(&node) {
            self.results.push(node);
            for child in &content.children {
                child.accept(self);
            }
        }
    }

    fn visit_concat(&mut self, concat: Rc<BibtexConcat>) {
        let node = BibtexNode::Content(BibtexContent::Concat(Rc::clone(&concat)));
        if self.check_range(&node) {
            self.results.push(node);
            concat.left.accept(self);
            if let Some(ref right) = concat.right {
                right.accept(self);
            }
        }
    }
}

pub struct BibtexSyntaxTree {
    pub root: BibtexRoot,
    pub descendants: Vec<BibtexNode>,
}

impl From<BibtexRoot> for BibtexSyntaxTree {
    fn from(root: BibtexRoot) -> Self {
        let mut finder = BibtexFinder::new(None);
        for child in &root.children {
            child.accept(&mut finder);
        }
        BibtexSyntaxTree {
            root,
            descendants: finder.results,
        }
    }
}

impl From<&str> for BibtexSyntaxTree {
    fn from(text: &str) -> Self {
        let tokens = BibtexLexer::from(text);
        let mut parser = BibtexParser::new(tokens);
        let root = parser.root();
        BibtexSyntaxTree::from(root)
    }
}

impl BibtexSyntaxTree {
    fn find(&self, position: Position) -> Vec<BibtexNode> {
        let mut finder = BibtexFinder::new(Some(position));
        for child in &self.root.children {
            child.accept(&mut finder);
        }
        finder.results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn verify(text: &str, expected: Vec<BibtexNodeKind>) {
        let actual: Vec<BibtexNodeKind> = BibtexSyntaxTree::from(text)
            .descendants
            .iter()
            .map(|node| node.kind())
            .collect();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_empty() {
        verify("", Vec::new());
    }

    #[test]
    fn test_preamble() {
        verify("@preamble", vec![BibtexNodeKind::Preamble]);
        verify("@preamble{", vec![BibtexNodeKind::Preamble]);
        verify(
            "@preamble{\"foo\"",
            vec![
                BibtexNodeKind::Preamble,
                BibtexNodeKind::QuotedContent,
                BibtexNodeKind::Word,
            ],
        );
        verify(
            "@preamble{\"foo\"}",
            vec![
                BibtexNodeKind::Preamble,
                BibtexNodeKind::QuotedContent,
                BibtexNodeKind::Word,
            ],
        );
    }

    #[test]
    fn test_string() {
        verify("@string", vec![BibtexNodeKind::String]);
        verify("@string{", vec![BibtexNodeKind::String]);
        verify("@string{key", vec![BibtexNodeKind::String]);
        verify(
            "@string{key=value",
            vec![BibtexNodeKind::String, BibtexNodeKind::Word],
        );
        verify(
            "@string{key=value}",
            vec![BibtexNodeKind::String, BibtexNodeKind::Word],
        );
    }

    #[test]
    fn test_entry() {
        verify("@article", vec![BibtexNodeKind::Entry]);
        verify("@article{", vec![BibtexNodeKind::Entry]);
        verify("@article{key", vec![BibtexNodeKind::Entry]);
        verify("@article{key,", vec![BibtexNodeKind::Entry]);
        verify(
            "@article{key, foo = bar}",
            vec![
                BibtexNodeKind::Entry,
                BibtexNodeKind::Field,
                BibtexNodeKind::Word,
            ],
        );
    }

    #[test]
    fn test_content() {
        verify(
            "@article{key, foo = {bar baz \\qux}}",
            vec![
                BibtexNodeKind::Entry,
                BibtexNodeKind::Field,
                BibtexNodeKind::BracedContent,
                BibtexNodeKind::Word,
                BibtexNodeKind::Word,
                BibtexNodeKind::Command,
            ],
        );
        verify(
            "@article{key, foo = bar # baz}",
            vec![
                BibtexNodeKind::Entry,
                BibtexNodeKind::Field,
                BibtexNodeKind::Concat,
                BibtexNodeKind::Word,
                BibtexNodeKind::Word,
            ],
        );
    }
}
