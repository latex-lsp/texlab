pub mod ast;
pub mod lexer;
pub mod parser;

use crate::range;
use crate::syntax::bibtex::ast::*;
use crate::syntax::bibtex::lexer::BibtexLexer;
use crate::syntax::bibtex::parser::BibtexParser;
use crate::syntax::text::SyntaxNode;
use lsp_types::{Position, Range};
use std::rc::Rc;

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

impl SyntaxNode for BibtexNode {
    fn range(&self) -> Range {
        match self {
            BibtexNode::Declaration(declaration) => declaration.range(),
            BibtexNode::Field(field) => field.range,
            BibtexNode::Content(content) => content.range(),
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
