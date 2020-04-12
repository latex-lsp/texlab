use crate::{Span, SyntaxNode};
use itertools::Itertools;
use petgraph::graph::{Graph, NodeIndex};
use serde::{Deserialize, Serialize};
use std::fmt;
use texlab_protocol::{Position, Range, RangeExt};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum TokenKind {
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

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Token {
    pub span: Span,
    pub kind: TokenKind,
}

impl SyntaxNode for Token {
    fn range(&self) -> Range {
        self.span.range()
    }
}

impl Token {
    pub fn new(span: Span, kind: TokenKind) -> Self {
        Self { span, kind }
    }

    pub fn text(&self) -> &str {
        &self.span.text
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub struct Root {
    pub range: Range,
}

impl SyntaxNode for Root {
    fn range(&self) -> Range {
        self.range
    }
}

impl fmt::Debug for Root {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Root")
    }
}

#[derive(PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Comment {
    pub token: Token,
}

impl SyntaxNode for Comment {
    fn range(&self) -> Range {
        self.token.range()
    }
}

impl fmt::Debug for Comment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Comment({})", self.token.text())
    }
}

#[derive(PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Preamble {
    pub range: Range,
    pub ty: Token,
    pub left: Option<Token>,
    pub right: Option<Token>,
}

impl SyntaxNode for Preamble {
    fn range(&self) -> Range {
        self.range
    }
}

impl fmt::Debug for Preamble {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Preamble")
    }
}

#[derive(PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct String {
    pub range: Range,
    pub ty: Token,
    pub left: Option<Token>,
    pub name: Option<Token>,
    pub assign: Option<Token>,
    pub right: Option<Token>,
}

impl SyntaxNode for String {
    fn range(&self) -> Range {
        self.range
    }
}

impl fmt::Debug for String {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "String({:?})", self.name.as_ref().map(Token::text))
    }
}

#[derive(PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub range: Range,
    pub ty: Token,
    pub left: Option<Token>,
    pub key: Option<Token>,
    pub comma: Option<Token>,
    pub right: Option<Token>,
}

impl SyntaxNode for Entry {
    fn range(&self) -> Range {
        self.range
    }
}

impl fmt::Debug for Entry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Entry({:?})", self.key.as_ref().map(Token::text))
    }
}

impl Entry {
    pub fn is_comment(&self) -> bool {
        self.ty.text().to_lowercase() == "@comment"
    }
}

#[derive(PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Field {
    pub range: Range,
    pub name: Token,
    pub assign: Option<Token>,
    pub comma: Option<Token>,
}

impl SyntaxNode for Field {
    fn range(&self) -> Range {
        self.range
    }
}

impl fmt::Debug for Field {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Field({})", self.name.text())
    }
}

#[derive(PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Word {
    pub token: Token,
}

impl SyntaxNode for Word {
    fn range(&self) -> Range {
        self.token.range()
    }
}

impl fmt::Debug for Word {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Word({})", self.token.text())
    }
}

#[derive(PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Command {
    pub token: Token,
}

impl SyntaxNode for Command {
    fn range(&self) -> Range {
        self.token.range()
    }
}

impl fmt::Debug for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Command({})", self.token.text())
    }
}

#[derive(PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct QuotedContent {
    pub range: Range,
    pub left: Token,
    pub right: Option<Token>,
}

impl SyntaxNode for QuotedContent {
    fn range(&self) -> Range {
        self.range
    }
}

impl fmt::Debug for QuotedContent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "QuotedContent")
    }
}

#[derive(PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct BracedContent {
    pub range: Range,
    pub left: Token,
    pub right: Option<Token>,
}

impl SyntaxNode for BracedContent {
    fn range(&self) -> Range {
        self.range
    }
}

impl fmt::Debug for BracedContent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BracedContent")
    }
}

#[derive(PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Concat {
    pub range: Range,
    pub operator: Token,
}

impl SyntaxNode for Concat {
    fn range(&self) -> Range {
        self.range
    }
}

impl fmt::Debug for Concat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Concat")
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum Node {
    Root(Root),
    Comment(Comment),
    Preamble(Box<Preamble>),
    String(Box<String>),
    Entry(Box<Entry>),
    Field(Box<Field>),
    Word(Word),
    Command(Command),
    QuotedContent(QuotedContent),
    BracedContent(BracedContent),
    Concat(Concat),
}

impl SyntaxNode for Node {
    fn range(&self) -> Range {
        match self {
            Self::Root(root) => root.range(),
            Self::Comment(comment) => comment.range(),
            Self::Preamble(preamble) => preamble.range(),
            Self::String(string) => string.range(),
            Self::Entry(entry) => entry.range(),
            Self::Field(field) => field.range(),
            Self::Word(word) => word.range(),
            Self::Command(cmd) => cmd.range(),
            Self::QuotedContent(content) => content.range(),
            Self::BracedContent(content) => content.range(),
            Self::Concat(concat) => concat.range(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tree {
    pub graph: Graph<Node, ()>,
    pub root: NodeIndex,
}

impl Tree {
    pub fn children(&self, parent: NodeIndex) -> impl Iterator<Item = NodeIndex> {
        self.graph
            .neighbors(parent)
            .sorted_by_key(|child| self.graph[*child].start())
    }

    pub fn has_children(&self, parent: NodeIndex) -> bool {
        self.children(parent).next().is_some()
    }

    pub fn walk<'a, V: Visitor<'a>>(&'a self, visitor: &mut V, parent: NodeIndex) {
        for child in self.children(parent) {
            visitor.visit(self, child);
        }
    }

    pub fn find(&self, pos: Position) -> Vec<NodeIndex> {
        let mut finder = Finder::new(pos);
        finder.visit(self, self.root);
        finder.results
    }

    pub fn as_preamble(&self, node: NodeIndex) -> Option<&Preamble> {
        if let Node::Preamble(preamble) = &self.graph[node] {
            Some(preamble)
        } else {
            None
        }
    }

    pub fn as_string(&self, node: NodeIndex) -> Option<&String> {
        if let Node::String(string) = &self.graph[node] {
            Some(string)
        } else {
            None
        }
    }

    pub fn as_entry(&self, node: NodeIndex) -> Option<&Entry> {
        if let Node::Entry(entry) = &self.graph[node] {
            Some(entry)
        } else {
            None
        }
    }

    pub fn as_field(&self, node: NodeIndex) -> Option<&Field> {
        if let Node::Field(field) = &self.graph[node] {
            Some(field)
        } else {
            None
        }
    }

    pub fn as_command(&self, node: NodeIndex) -> Option<&Command> {
        if let Node::Command(cmd) = &self.graph[node] {
            Some(cmd)
        } else {
            None
        }
    }

    pub fn as_word(&self, node: NodeIndex) -> Option<&Word> {
        if let Node::Word(word) = &self.graph[node] {
            Some(word)
        } else {
            None
        }
    }

    pub fn entry_by_key(&self, key: &str) -> Option<NodeIndex> {
        for node in self.children(self.root) {
            if let Some(entry) = self.as_entry(node) {
                if entry.key.as_ref().map(Token::text) == Some(key) {
                    return Some(node);
                }
            }
        }
        None
    }

    pub fn field_by_name(&self, parent: NodeIndex, name: &str) -> Option<NodeIndex> {
        let name = name.to_lowercase();
        self.as_entry(parent)?;
        for node in self.children(parent) {
            if let Some(field) = self.as_field(node) {
                if field.name.text() == name {
                    return Some(node);
                }
            }
        }
        None
    }

    pub fn crossref(&self, entry: NodeIndex) -> Option<NodeIndex> {
        let field = self.field_by_name(entry, "crossref")?;
        let content = self.children(field).next()?;
        let key = self.as_word(self.children(content).next()?)?;
        self.entry_by_key(key.token.text())
    }
}

pub trait Visitor<'a> {
    fn visit(&mut self, tree: &'a Tree, node: NodeIndex);
}

#[derive(Debug)]
struct Finder {
    position: Position,
    results: Vec<NodeIndex>,
}

impl Finder {
    fn new(position: Position) -> Self {
        Self {
            position,
            results: Vec::new(),
        }
    }
}

impl<'a> Visitor<'a> for Finder {
    fn visit(&mut self, tree: &'a Tree, node: NodeIndex) {
        if tree.graph[node].range().contains(self.position) {
            self.results.push(node);
            tree.walk(self, node);
        }
    }
}
