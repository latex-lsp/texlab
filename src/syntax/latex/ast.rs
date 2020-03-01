use crate::{
    protocol::{Position, Range, RangeExt},
    syntax::text::{Span, SyntaxNode},
};
use itertools::Itertools;
use petgraph::graph::{Graph, NodeIndex};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum LatexTokenKind {
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
pub struct LatexToken {
    pub span: Span,
    pub kind: LatexTokenKind,
}

impl LatexToken {
    pub fn new(span: Span, kind: LatexTokenKind) -> Self {
        Self { span, kind }
    }

    pub fn text(&self) -> &str {
        &self.span.text
    }
}

impl SyntaxNode for LatexToken {
    fn range(&self) -> Range {
        self.span.range()
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub struct LatexRoot {
    pub range: Range,
}

impl SyntaxNode for LatexRoot {
    fn range(&self) -> Range {
        self.range
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum LatexGroupKind {
    Group,
    Options,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct LatexGroup {
    pub range: Range,
    pub left: LatexToken,
    pub right: Option<LatexToken>,
    pub kind: LatexGroupKind,
}

impl SyntaxNode for LatexGroup {
    fn range(&self) -> Range {
        self.range
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct LatexCommand {
    pub range: Range,
    pub name: LatexToken,
}

impl LatexCommand {
    pub fn short_name_range(&self) -> Range {
        Range::new_simple(
            self.name.start().line,
            self.name.start().character + 1,
            self.name.end().line,
            self.name.end().character,
        )
    }
}

impl SyntaxNode for LatexCommand {
    fn range(&self) -> Range {
        self.range
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct LatexText {
    pub range: Range,
    pub words: Vec<LatexToken>,
}

impl SyntaxNode for LatexText {
    fn range(&self) -> Range {
        self.range
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct LatexComma {
    pub range: Range,
    pub token: LatexToken,
}

impl SyntaxNode for LatexComma {
    fn range(&self) -> Range {
        self.range
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct LatexMath {
    pub range: Range,
    pub token: LatexToken,
}

impl SyntaxNode for LatexMath {
    fn range(&self) -> Range {
        self.range
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum LatexNode {
    Root(LatexRoot),
    Group(LatexGroup),
    Command(LatexCommand),
    Text(LatexText),
    Comma(LatexComma),
    Math(LatexMath),
}

impl LatexNode {
    pub fn as_command(&self) -> Option<&LatexCommand> {
        if let Self::Command(cmd) = self {
            Some(cmd)
        } else {
            None
        }
    }
}

impl SyntaxNode for LatexNode {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatexTree {
    pub graph: Graph<LatexNode, ()>,
    pub root: NodeIndex,
}

impl LatexTree {
    pub fn children(&self, parent: NodeIndex) -> impl Iterator<Item = NodeIndex> {
        self.graph
            .neighbors(parent)
            .sorted_by_key(|child| self.graph.node_weight(*child).unwrap().start())
    }

    pub fn walk<V: LatexVisitor>(&self, visitor: &mut V, parent: NodeIndex) {
        for child in self.children(parent) {
            visitor.visit(self, child);
        }
    }

    pub fn find(&self, positon: Position) -> Vec<NodeIndex> {
        let mut finder = LatexFinder::new(positon);
        finder.visit(self, self.root);
        finder.results
    }

    pub fn print(&self, node: NodeIndex) -> String {
        let start_position = self.graph.node_weight(node).unwrap().start();
        let mut printer = LatexPrinter::new(start_position);
        printer.visit(self, node);
        printer.output
    }
}

pub trait LatexVisitor {
    fn visit(&mut self, tree: &LatexTree, node: NodeIndex);
}

#[derive(Debug)]
struct LatexFinder {
    position: Position,
    results: Vec<NodeIndex>,
}

impl LatexFinder {
    fn new(position: Position) -> Self {
        Self {
            position,
            results: Vec::new(),
        }
    }
}

impl LatexVisitor for LatexFinder {
    fn visit(&mut self, tree: &LatexTree, node: NodeIndex) {
        if tree
            .graph
            .node_weight(node)
            .unwrap()
            .range()
            .contains(self.position)
        {
            self.results.push(node);
            tree.walk(self, node);
        }
    }
}

#[derive(Debug)]
struct LatexPrinter {
    output: String,
    position: Position,
}

impl LatexPrinter {
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

    fn print_token(&mut self, token: &LatexToken) {
        self.synchronize(token.start());
        self.output.push_str(token.text());
        self.position.character += token.end().character - token.start().character;
        self.synchronize(token.end());
    }
}

impl LatexVisitor for LatexPrinter {
    fn visit(&mut self, tree: &LatexTree, node: NodeIndex) {
        match tree.graph.node_weight(node).unwrap() {
            LatexNode::Root(_) => tree.walk(self, node),
            LatexNode::Group(group) => {
                self.print_token(&group.left);
                tree.walk(self, node);
                if let Some(right) = &group.right {
                    self.print_token(right);
                }
            }
            LatexNode::Command(cmd) => {
                self.print_token(&cmd.name);
                tree.walk(self, node);
            }
            LatexNode::Text(text) => {
                for word in &text.words {
                    self.print_token(word);
                }
            }
            LatexNode::Comma(comma) => {
                self.print_token(&comma.token);
            }
            LatexNode::Math(math) => {
                self.print_token(&math.token);
            }
        }
    }
}
