use super::ast::*;
use crate::text::SyntaxNode;
use petgraph::graph::{Graph, NodeIndex};
use std::iter::Peekable;
use texlab_protocol::{Range, RangeExt};

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
        let mut children = Vec::new();

        while let Some(ref token) = self.tokens.peek() {
            match token.kind {
                TokenKind::PreambleKind => children.push(self.preamble()),
                TokenKind::StringKind => children.push(self.string()),
                TokenKind::EntryKind => children.push(self.entry()),
                _ => children.push(self.comment()),
            }
        }

        let range = if children.is_empty() {
            Range::new_simple(0, 0, 0, 0)
        } else {
            let start = self.graph[children[0]].start();
            let end = self.graph[children[children.len() - 1]].end();
            Range::new(start, end)
        };

        let root = self.graph.add_node(Node::Root(Root { range }));
        self.connect(root, &children);
        Tree {
            graph: self.graph,
            root,
        }
    }

    fn preamble(&mut self) -> NodeIndex {
        let ty = self.tokens.next().unwrap();

        let left = self.expect2(TokenKind::BeginBrace, TokenKind::BeginParen);
        if left.is_none() {
            return self.graph.add_node(Node::Preamble(Box::new(Preamble {
                range: ty.range(),
                ty,
                left: None,
                right: None,
            })));
        }

        if !self.can_match_content() {
            return self.graph.add_node(Node::Preamble(Box::new(Preamble {
                range: Range::new(ty.start(), left.as_ref().unwrap().end()),
                ty,
                left,
                right: None,
            })));
        }

        let content = self.content();

        let right = self.expect2(TokenKind::EndBrace, TokenKind::EndParen);
        let end = right
            .as_ref()
            .map(Token::end)
            .unwrap_or_else(|| self.graph[content].end());

        let parent = self.graph.add_node(Node::Preamble(Box::new(Preamble {
            range: Range::new(ty.start(), end),
            ty,
            left,
            right,
        })));
        self.graph.add_edge(parent, content, ());
        parent
    }

    fn string(&mut self) -> NodeIndex {
        let ty = self.tokens.next().unwrap();

        let left = self.expect2(TokenKind::BeginBrace, TokenKind::BeginParen);
        if left.is_none() {
            return self.graph.add_node(Node::String(Box::new(String {
                range: ty.range(),
                ty,
                left: None,
                name: None,
                assign: None,
                right: None,
            })));
        }

        let name = self.expect1(TokenKind::Word);
        if name.is_none() {
            return self.graph.add_node(Node::String(Box::new(String {
                range: Range::new(ty.start(), left.as_ref().unwrap().end()),
                ty,
                left,
                name: None,
                assign: None,
                right: None,
            })));
        }

        let assign = self.expect1(TokenKind::Assign);
        if assign.is_none() {
            return self.graph.add_node(Node::String(Box::new(String {
                range: Range::new(ty.start(), name.as_ref().unwrap().end()),
                ty,
                left,
                name,
                assign: None,
                right: None,
            })));
        }

        if !self.can_match_content() {
            return self.graph.add_node(Node::String(Box::new(String {
                range: Range::new(ty.start(), assign.as_ref().unwrap().end()),
                ty,
                left,
                name,
                assign,
                right: None,
            })));
        }
        let value = self.content();

        let right = self.expect2(TokenKind::EndBrace, TokenKind::EndParen);
        let end = right
            .as_ref()
            .map(Token::end)
            .unwrap_or_else(|| self.graph[value].end());

        let parent = self.graph.add_node(Node::String(Box::new(String {
            range: Range::new(ty.start(), end),
            ty,
            left,
            name,
            assign,
            right,
        })));
        self.graph.add_edge(parent, value, ());
        parent
    }

    fn entry(&mut self) -> NodeIndex {
        let ty = self.tokens.next().unwrap();

        let left = self.expect2(TokenKind::BeginBrace, TokenKind::BeginParen);
        if left.is_none() {
            return self.graph.add_node(Node::Entry(Box::new(Entry {
                range: ty.range(),
                ty,
                left: None,
                key: None,
                comma: None,
                right: None,
            })));
        }

        let key = self.expect1(TokenKind::Word);
        if key.is_none() {
            return self.graph.add_node(Node::Entry(Box::new(Entry {
                range: Range::new(ty.start(), left.as_ref().unwrap().end()),
                ty,
                left,
                key: None,
                comma: None,
                right: None,
            })));
        }

        let comma = self.expect1(TokenKind::Comma);
        if comma.is_none() {
            return self.graph.add_node(Node::Entry(Box::new(Entry {
                range: Range::new(ty.start(), key.as_ref().unwrap().end()),
                ty,
                left,
                key,
                comma: None,
                right: None,
            })));
        }

        let mut fields = Vec::new();
        while self.next_of_kind(TokenKind::Word) {
            fields.push(self.field());
        }

        let right = self.expect2(TokenKind::EndBrace, TokenKind::EndParen);

        let end = right
            .as_ref()
            .map(Token::end)
            .or_else(|| fields.last().map(|field| self.graph[*field].end()))
            .unwrap_or_else(|| comma.as_ref().unwrap().end());
        let parent = self.graph.add_node(Node::Entry(Box::new(Entry {
            range: Range::new(ty.start(), end),
            ty,
            left,
            key,
            comma,
            right,
        })));
        self.connect(parent, &fields);
        parent
    }

    fn comment(&mut self) -> NodeIndex {
        let token = self.tokens.next().unwrap();
        self.graph.add_node(Node::Comment(Comment { token }))
    }

    fn field(&mut self) -> NodeIndex {
        let name = self.tokens.next().unwrap();

        let assign = self.expect1(TokenKind::Assign);
        if assign.is_none() {
            return self.graph.add_node(Node::Field(Box::new(Field {
                range: name.range(),
                name,
                assign: None,
                comma: None,
            })));
        }

        if !self.can_match_content() {
            return self.graph.add_node(Node::Field(Box::new(Field {
                range: Range::new(name.start(), assign.as_ref().unwrap().end()),
                name,
                assign,
                comma: None,
            })));
        }

        let content = self.content();

        let comma = self.expect1(TokenKind::Comma);

        let end = comma
            .as_ref()
            .map(Token::end)
            .unwrap_or_else(|| self.graph[content].end());
        let parent = self.graph.add_node(Node::Field(Box::new(Field {
            range: Range::new(name.start(), end),
            name,
            assign,
            comma,
        })));
        self.graph.add_edge(parent, content, ());
        parent
    }

    fn content(&mut self) -> NodeIndex {
        let token = self.tokens.next().unwrap();
        let left = match token.kind {
            TokenKind::PreambleKind
            | TokenKind::StringKind
            | TokenKind::EntryKind
            | TokenKind::Word
            | TokenKind::Assign
            | TokenKind::Comma
            | TokenKind::BeginParen
            | TokenKind::EndParen => self.graph.add_node(Node::Word(Word { token })),
            TokenKind::Command => self.graph.add_node(Node::Command(Command { token })),
            TokenKind::Quote => {
                let mut children = Vec::new();
                while self.can_match_content() {
                    if self.next_of_kind(TokenKind::Quote) {
                        break;
                    }
                    children.push(self.content());
                }
                let right = self.expect1(TokenKind::Quote);

                let end = right
                    .as_ref()
                    .map(Token::end)
                    .or_else(|| children.last().map(|child| self.graph[*child].end()))
                    .unwrap_or_else(|| token.end());
                let parent = self.graph.add_node(Node::QuotedContent(QuotedContent {
                    range: Range::new(token.start(), end),
                    left: token,
                    right,
                }));
                self.connect(parent, &children);
                parent
            }
            TokenKind::BeginBrace => {
                let mut children = Vec::new();
                while self.can_match_content() {
                    children.push(self.content());
                }
                let right = self.expect1(TokenKind::EndBrace);

                let end = right
                    .as_ref()
                    .map(Token::end)
                    .or_else(|| children.last().map(|child| self.graph[*child].end()))
                    .unwrap_or_else(|| token.end());
                let parent = self.graph.add_node(Node::BracedContent(BracedContent {
                    range: Range::new(token.start(), end),
                    left: token,
                    right,
                }));
                self.connect(parent, &children);
                parent
            }
            _ => unreachable!(),
        };

        match self.expect1(TokenKind::Concat) {
            Some(operator) => {
                if self.can_match_content() {
                    let right = self.content();
                    let parent = self.graph.add_node(Node::Concat(Concat {
                        range: Range::new(self.graph[left].start(), self.graph[right].end()),
                        operator,
                    }));
                    self.graph.add_edge(parent, left, ());
                    self.graph.add_edge(parent, right, ());
                    parent
                } else {
                    let parent = self.graph.add_node(Node::Concat(Concat {
                        range: Range::new(self.graph[left].start(), operator.end()),
                        operator,
                    }));
                    self.graph.add_edge(parent, left, ());
                    parent
                }
            }
            None => left,
        }
    }

    fn connect(&mut self, parent: NodeIndex, children: &[NodeIndex]) {
        for child in children {
            self.graph.add_edge(parent, *child, ());
        }
    }

    fn can_match_content(&mut self) -> bool {
        if let Some(ref token) = self.tokens.peek() {
            match token.kind {
                TokenKind::PreambleKind
                | TokenKind::StringKind
                | TokenKind::EntryKind
                | TokenKind::Word
                | TokenKind::Command
                | TokenKind::Assign
                | TokenKind::Comma
                | TokenKind::Quote
                | TokenKind::BeginBrace
                | TokenKind::BeginParen
                | TokenKind::EndParen => true,
                TokenKind::Concat | TokenKind::EndBrace => false,
            }
        } else {
            false
        }
    }

    fn expect1(&mut self, kind: TokenKind) -> Option<Token> {
        if let Some(ref token) = self.tokens.peek() {
            if token.kind == kind {
                return self.tokens.next();
            }
        }
        None
    }

    fn expect2(&mut self, kind1: TokenKind, kind2: TokenKind) -> Option<Token> {
        if let Some(ref token) = self.tokens.peek() {
            if token.kind == kind1 || token.kind == kind2 {
                return self.tokens.next();
            }
        }
        None
    }

    fn next_of_kind(&mut self, kind: TokenKind) -> bool {
        if let Some(token) = self.tokens.peek() {
            token.kind == kind
        } else {
            false
        }
    }
}
