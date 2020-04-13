use crate::{
    generic_ast::{Ast, AstNodeIndex},
    text::{Span, SyntaxNode},
};
use serde::{Deserialize, Serialize};
use std::ops::Deref;
use texlab_protocol::{Position, Range, RangeExt};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum TokenKind {
    Word,
    Command,
    Math,
    Comma,
    BeginGroup,
    EndGroup,
    BeginOptions,
    EndOptions,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Token {
    pub span: Span,
    pub kind: TokenKind,
}

impl Token {
    pub fn new(span: Span, kind: TokenKind) -> Self {
        Self { span, kind }
    }

    pub fn text(&self) -> &str {
        &self.span.text
    }
}

impl SyntaxNode for Token {
    fn range(&self) -> Range {
        self.span.range()
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub struct Root {
    pub range: Range,
}

impl SyntaxNode for Root {
    fn range(&self) -> Range {
        self.range
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum GroupKind {
    Group,
    Options,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Group {
    pub range: Range,
    pub left: Token,
    pub right: Option<Token>,
    pub kind: GroupKind,
}

impl SyntaxNode for Group {
    fn range(&self) -> Range {
        self.range
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Command {
    pub range: Range,
    pub name: Token,
}

impl Command {
    pub fn short_name_range(&self) -> Range {
        Range::new_simple(
            self.name.start().line,
            self.name.start().character + 1,
            self.name.end().line,
            self.name.end().character,
        )
    }
}

impl SyntaxNode for Command {
    fn range(&self) -> Range {
        self.range
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Text {
    pub range: Range,
    pub words: Vec<Token>,
}

impl SyntaxNode for Text {
    fn range(&self) -> Range {
        self.range
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Comma {
    pub range: Range,
    pub token: Token,
}

impl SyntaxNode for Comma {
    fn range(&self) -> Range {
        self.range
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Math {
    pub range: Range,
    pub token: Token,
}

impl SyntaxNode for Math {
    fn range(&self) -> Range {
        self.range
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum Node {
    Root(Root),
    Group(Group),
    Command(Command),
    Text(Text),
    Comma(Comma),
    Math(Math),
}

impl SyntaxNode for Node {
    fn range(&self) -> Range {
        match self {
            Self::Root(root) => root.range(),
            Self::Group(group) => group.range(),
            Self::Command(cmd) => cmd.range(),
            Self::Text(text) => text.range(),
            Self::Comma(comma) => comma.range(),
            Self::Math(math) => math.range(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Tree {
    pub inner: Ast<Node>,
    pub root: AstNodeIndex,
}

impl Deref for Tree {
    type Target = Ast<Node>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Tree {
    pub fn walk<V: Visitor>(&self, visitor: &mut V, parent: AstNodeIndex) {
        for child in self.children(parent) {
            visitor.visit(self, child);
        }
    }

    pub fn find(&self, pos: Position) -> Vec<AstNodeIndex> {
        let mut finder = Finder::new(pos);
        finder.visit(self, self.root);
        finder.results
    }

    pub fn find_command_by_short_name_range(&self, pos: Position) -> Option<AstNodeIndex> {
        self.find(pos).into_iter().find(|node| {
            self.as_command(*node)
                .filter(|cmd| {
                    cmd.name.range().contains(pos) && cmd.name.start().character != pos.character
                })
                .is_some()
        })
    }

    pub fn print(&self, node: AstNodeIndex) -> String {
        let start_position = self[node].start();
        let mut printer = Printer::new(start_position);
        printer.visit(self, node);
        printer.output
    }

    pub fn commands<'a>(&'a self) -> impl Iterator<Item = AstNodeIndex> + 'a {
        self.inner
            .nodes()
            .into_iter()
            .filter(move |node| self.as_command(*node).is_some())
    }

    pub fn as_group(&self, node: AstNodeIndex) -> Option<&Group> {
        if let Node::Group(group) = &self[node] {
            Some(group)
        } else {
            None
        }
    }

    pub fn as_command(&self, node: AstNodeIndex) -> Option<&Command> {
        if let Node::Command(cmd) = &self[node] {
            Some(cmd)
        } else {
            None
        }
    }

    pub fn as_text(&self, node: AstNodeIndex) -> Option<&Text> {
        if let Node::Text(text) = &self[node] {
            Some(text)
        } else {
            None
        }
    }

    pub fn as_math(&self, node: AstNodeIndex) -> Option<&Math> {
        if let Node::Math(math) = &self[node] {
            Some(math)
        } else {
            None
        }
    }

    pub fn extract_group(
        &self,
        parent: AstNodeIndex,
        group_kind: GroupKind,
        index: usize,
    ) -> Option<AstNodeIndex> {
        self.children(parent)
            .filter(|child| {
                self.as_group(*child)
                    .filter(|group| group.kind == group_kind)
                    .is_some()
            })
            .nth(index)
    }

    pub fn extract_text(
        &self,
        parent: AstNodeIndex,
        group_kind: GroupKind,
        index: usize,
    ) -> Option<&Text> {
        let group = self.extract_group(parent, group_kind, index)?;
        let mut contents = self.children(group);
        let text = self.as_text(contents.next()?);
        if contents.next().is_none() {
            text
        } else {
            None
        }
    }

    pub fn extract_word(
        &self,
        parent: AstNodeIndex,
        group_kind: GroupKind,
        index: usize,
    ) -> Option<&Token> {
        let text = self.extract_text(parent, group_kind, index)?;
        if text.words.len() == 1 {
            Some(&text.words[0])
        } else {
            None
        }
    }

    pub fn extract_comma_separated_words(
        &self,
        parent: AstNodeIndex,
        group_kind: GroupKind,
        index: usize,
    ) -> Option<Vec<&Token>> {
        let group = self.extract_group(parent, group_kind, index)?;
        let mut words = Vec::new();
        for child in self.children(group) {
            match &self[child] {
                Node::Root(_) | Node::Group(_) | Node::Command(_) | Node::Math(_) => return None,
                Node::Text(text) => {
                    for word in &text.words {
                        words.push(word);
                    }
                }
                Node::Comma(_) => (),
            }
        }
        Some(words)
    }

    pub fn print_group_content(
        &self,
        parent: AstNodeIndex,
        group_kind: GroupKind,
        index: usize,
    ) -> Option<String> {
        let arg = self.extract_group(parent, group_kind, index)?;
        let text = self.print(arg);
        self.as_group(arg)?.right.as_ref()?;
        Some(text[1..text.len() - 1].trim().into())
    }
}

pub trait Visitor {
    fn visit(&mut self, tree: &Tree, node: AstNodeIndex);
}

#[derive(Debug)]
struct Finder {
    position: Position,
    results: Vec<AstNodeIndex>,
}

impl Finder {
    fn new(position: Position) -> Self {
        Self {
            position,
            results: Vec::new(),
        }
    }
}

impl Visitor for Finder {
    fn visit(&mut self, tree: &Tree, node: AstNodeIndex) {
        if tree[node].range().contains(self.position) {
            self.results.push(node);
            tree.walk(self, node);
        }
    }
}

#[derive(Debug)]
struct Printer {
    output: String,
    position: Position,
}

impl Printer {
    fn new(start_position: Position) -> Self {
        Self {
            output: String::new(),
            position: start_position,
        }
    }

    fn synchronize(&mut self, position: Position) {
        while self.position.line < position.line {
            self.output.push('\n');
            self.position.line += 1;
            self.position.character = 0;
        }

        while self.position.character < position.character {
            self.output.push(' ');
            self.position.character += 1;
        }

        assert_eq!(self.position, position);
    }

    fn print_token(&mut self, token: &Token) {
        self.synchronize(token.start());
        self.output.push_str(token.text());
        self.position.character += token.end().character - token.start().character;
        self.synchronize(token.end());
    }
}

impl Visitor for Printer {
    fn visit(&mut self, tree: &Tree, node: AstNodeIndex) {
        match &tree[node] {
            Node::Root(_) => tree.walk(self, node),
            Node::Group(group) => {
                self.print_token(&group.left);
                tree.walk(self, node);
                if let Some(right) = &group.right {
                    self.print_token(right);
                }
            }
            Node::Command(cmd) => {
                self.print_token(&cmd.name);
                tree.walk(self, node);
            }
            Node::Text(text) => {
                for word in &text.words {
                    self.print_token(word);
                }
            }
            Node::Comma(comma) => {
                self.print_token(&comma.token);
            }
            Node::Math(math) => {
                self.print_token(&math.token);
            }
        }
    }
}
