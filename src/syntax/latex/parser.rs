use super::ast::*;
use crate::{
    protocol::{Range, RangeExt},
    syntax::text::SyntaxNode,
};
use petgraph::graph::{Graph, NodeIndex};
use std::iter::Peekable;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Scope {
    Root,
    Group,
    Options,
}

#[derive(Debug)]
pub struct Parser<I: Iterator<Item = Token>> {
    graph: Graph<Node, ()>,
    tokens: Peekable<I>,
}

impl<I: Iterator<Item = Token>> Parser<I> {
    pub fn new(tokens: I) -> Self {
        Self {
            tokens: tokens.peekable(),
            graph: Graph::new(),
        }
    }

    pub fn parse(mut self) -> Tree {
        let children = self.content(Scope::Root);

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

        let root = self.graph.add_node(Node::Root(Root { range }));
        self.connect(root, &children);
        Tree {
            graph: self.graph,
            root,
        }
    }

    fn content(&mut self, scope: Scope) -> Vec<NodeIndex> {
        let mut children = Vec::new();
        while let Some(ref token) = self.tokens.peek() {
            match token.kind {
                TokenKind::Word | TokenKind::BeginOptions => {
                    children.push(self.text(scope));
                }
                TokenKind::Command => {
                    children.push(self.command());
                }
                TokenKind::Comma => {
                    children.push(self.comma());
                }
                TokenKind::Math => {
                    children.push(self.math());
                }
                TokenKind::BeginGroup => {
                    children.push(self.group(GroupKind::Group));
                }
                TokenKind::EndGroup => {
                    if scope == Scope::Root {
                        self.tokens.next();
                    } else {
                        return children;
                    }
                }
                TokenKind::EndOptions => {
                    if scope == Scope::Options {
                        return children;
                    } else {
                        children.push(self.text(scope));
                    }
                }
            }
        }
        children
    }

    fn group(&mut self, kind: GroupKind) -> NodeIndex {
        let left = self.tokens.next().unwrap();
        let scope = match kind {
            GroupKind::Group => Scope::Group,
            GroupKind::Options => Scope::Options,
        };

        let children = self.content(scope);
        let right_kind = match kind {
            GroupKind::Group => TokenKind::EndGroup,
            GroupKind::Options => TokenKind::EndOptions,
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

        let node = self.graph.add_node(Node::Group(Group {
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
                TokenKind::BeginGroup => children.push(self.group(GroupKind::Group)),
                TokenKind::BeginOptions => children.push(self.group(GroupKind::Options)),
                _ => break,
            }
        }

        let end = children
            .last()
            .map(|child| self.graph.node_weight(*child).unwrap().end())
            .unwrap_or_else(|| name.end());
        let range = Range::new(name.start(), end);

        let node = self.graph.add_node(Node::Command(Command { range, name }));
        self.connect(node, &children);
        node
    }

    fn text(&mut self, scope: Scope) -> NodeIndex {
        let mut words = Vec::new();
        while let Some(ref token) = self.tokens.peek() {
            let kind = token.kind;
            let opts = kind == TokenKind::EndOptions && scope != Scope::Options;
            if kind == TokenKind::Word || kind == TokenKind::BeginOptions || opts {
                words.push(self.tokens.next().unwrap());
            } else {
                break;
            }
        }
        let range = Range::new(words[0].start(), words[words.len() - 1].end());
        self.graph.add_node(Node::Text(Text { range, words }))
    }

    fn comma(&mut self) -> NodeIndex {
        let token = self.tokens.next().unwrap();
        let range = token.range();
        self.graph.add_node(Node::Comma(Comma { range, token }))
    }

    fn math(&mut self) -> NodeIndex {
        let token = self.tokens.next().unwrap();
        let range = token.range();
        self.graph.add_node(Node::Math(Math { range, token }))
    }

    fn connect(&mut self, parent: NodeIndex, children: &[NodeIndex]) {
        for child in children {
            self.graph.add_edge(parent, *child, ());
        }
    }

    fn next_of_kind(&mut self, kind: TokenKind) -> bool {
        self.tokens
            .peek()
            .filter(|token| token.kind == kind)
            .is_some()
    }
}
