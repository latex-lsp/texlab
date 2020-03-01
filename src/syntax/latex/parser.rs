use super::ast::*;
use crate::{
    protocol::{Range, RangeExt},
    syntax::text::SyntaxNode,
};
use petgraph::graph::{Graph, NodeIndex};
use std::iter::Peekable;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum LatexScope {
    Root,
    Group,
    Options,
}

#[derive(Debug)]
pub struct LatexParser<I: Iterator<Item = LatexToken>> {
    graph: Graph<LatexNode, ()>,
    tokens: Peekable<I>,
}

impl<I: Iterator<Item = LatexToken>> LatexParser<I> {
    pub fn new(tokens: I) -> Self {
        Self {
            tokens: tokens.peekable(),
            graph: Graph::new(),
        }
    }

    pub fn parse(mut self) -> LatexTree {
        let children = self.content(LatexScope::Root);

        let range = if children.is_empty() {
            Range::new_simple(0, 0, 0, 0)
        } else {
            let start = self.graph.node_weight(children[0]).unwrap().start();
            let end = self
                .graph
                .node_weight(children[children.len() - 1])
                .unwrap()
                .end();
            Range::new(start, end)
        };

        let root = self.graph.add_node(LatexNode::Root(LatexRoot { range }));
        self.connect(root, &children);
        LatexTree {
            graph: self.graph,
            root,
        }
    }

    fn content(&mut self, scope: LatexScope) -> Vec<NodeIndex> {
        let mut children = Vec::new();
        while let Some(ref token) = self.tokens.peek() {
            match token.kind {
                LatexTokenKind::Word | LatexTokenKind::BeginOptions => {
                    children.push(self.text(scope));
                }
                LatexTokenKind::Command => {
                    children.push(self.command());
                }
                LatexTokenKind::Comma => {
                    children.push(self.comma());
                }
                LatexTokenKind::Math => {
                    children.push(self.math());
                }
                LatexTokenKind::BeginGroup => {
                    children.push(self.group(LatexGroupKind::Group));
                }
                LatexTokenKind::EndGroup => {
                    if scope == LatexScope::Root {
                        self.tokens.next();
                    } else {
                        return children;
                    }
                }
                LatexTokenKind::EndOptions => {
                    if scope == LatexScope::Options {
                        return children;
                    } else {
                        children.push(self.text(scope));
                    }
                }
            }
        }
        children
    }

    fn group(&mut self, kind: LatexGroupKind) -> NodeIndex {
        let left = self.tokens.next().unwrap();
        let scope = match kind {
            LatexGroupKind::Group => LatexScope::Group,
            LatexGroupKind::Options => LatexScope::Options,
        };

        let children = self.content(scope);
        let right_kind = match kind {
            LatexGroupKind::Group => LatexTokenKind::EndGroup,
            LatexGroupKind::Options => LatexTokenKind::EndOptions,
        };

        let right = if self.next_of_kind(right_kind) {
            self.tokens.next()
        } else {
            None
        };

        let end = right
            .as_ref()
            .map(SyntaxNode::end)
            .or_else(|| {
                children
                    .last()
                    .map(|child| self.graph.node_weight(*child).unwrap().end())
            })
            .unwrap_or_else(|| left.end());
        let range = Range::new(left.start(), end);

        let node = self.graph.add_node(LatexNode::Group(LatexGroup {
            range,
            left,
            kind,
            right,
        }));
        self.connect(node, &children);
        node
    }

    fn command(&mut self) -> NodeIndex {
        let name = self.tokens.next().unwrap();
        let mut children = Vec::new();
        while let Some(token) = self.tokens.peek() {
            match token.kind {
                LatexTokenKind::BeginGroup => children.push(self.group(LatexGroupKind::Group)),
                LatexTokenKind::BeginOptions => children.push(self.group(LatexGroupKind::Options)),
                _ => break,
            }
        }

        let end = children
            .last()
            .map(|child| self.graph.node_weight(*child).unwrap().end())
            .unwrap_or_else(|| name.end());
        let range = Range::new(name.start(), end);

        let node = self
            .graph
            .add_node(LatexNode::Command(LatexCommand { range, name }));
        self.connect(node, &children);
        node
    }

    fn text(&mut self, scope: LatexScope) -> NodeIndex {
        let mut words = Vec::new();
        while let Some(ref token) = self.tokens.peek() {
            let kind = token.kind;
            let opts = kind == LatexTokenKind::EndOptions && scope != LatexScope::Options;
            if kind == LatexTokenKind::Word || kind == LatexTokenKind::BeginOptions || opts {
                words.push(self.tokens.next().unwrap());
            } else {
                break;
            }
        }
        let range = Range::new(words[0].start(), words[words.len() - 1].end());
        self.graph
            .add_node(LatexNode::Text(LatexText { range, words }))
    }

    fn comma(&mut self) -> NodeIndex {
        let token = self.tokens.next().unwrap();
        let range = token.range();
        self.graph
            .add_node(LatexNode::Comma(LatexComma { range, token }))
    }

    fn math(&mut self) -> NodeIndex {
        let token = self.tokens.next().unwrap();
        let range = token.range();
        self.graph
            .add_node(LatexNode::Math(LatexMath { range, token }))
    }

    fn connect(&mut self, parent: NodeIndex, children: &[NodeIndex]) {
        for child in children {
            self.graph.add_edge(parent, *child, ());
        }
    }

    fn next_of_kind(&mut self, kind: LatexTokenKind) -> bool {
        self.tokens
            .peek()
            .filter(|token| token.kind == kind)
            .is_some()
    }
}
