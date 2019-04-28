use crate::range;
use crate::syntax::bibtex::ast::*;
use crate::syntax::text::SyntaxNode;
use lsp_types::Position;

pub enum BibtexNode<'a> {
    Root(&'a BibtexRoot),
    Preamble(&'a BibtexPreamble),
    String(&'a BibtexString),
    Entry(&'a BibtexEntry),
    Comment(&'a BibtexComment),
    Field(&'a BibtexField),
    Word(&'a BibtexWord),
    Command(&'a BibtexCommand),
    QuotedContent(&'a BibtexQuotedContent),
    BracedContent(&'a BibtexBracedContent),
    Concat(&'a BibtexConcat),
}

pub struct BibtexFinder<'a> {
    pub position: Position,
    pub results: Vec<BibtexNode<'a>>,
}

impl<'a> BibtexFinder<'a> {
    pub fn new(position: Position) -> Self {
        BibtexFinder {
            position,
            results: Vec::new(),
        }
    }
}

impl<'a> BibtexVisitor<'a> for BibtexFinder<'a> {
    fn visit_root(&mut self, root: &'a BibtexRoot) {
        if range::contains(root.range(), self.position) {
            self.results.push(BibtexNode::Root(root));
            BibtexWalker::walk_root(self, root);
        }
    }

    fn visit_comment(&mut self, comment: &'a BibtexComment) {
        if range::contains(comment.range(), self.position) {
            self.results.push(BibtexNode::Comment(comment));
        }
    }

    fn visit_preamble(&mut self, preamble: &'a BibtexPreamble) {
        if range::contains(preamble.range(), self.position) {
            self.results.push(BibtexNode::Preamble(preamble));
            BibtexWalker::walk_preamble(self, preamble);
        }
    }

    fn visit_string(&mut self, string: &'a BibtexString) {
        if range::contains(string.range(), self.position) {
            self.results.push(BibtexNode::String(string));
            BibtexWalker::walk_string(self, string);
        }
    }

    fn visit_entry(&mut self, entry: &'a BibtexEntry) {
        if range::contains(entry.range(), self.position) {
            self.results.push(BibtexNode::Entry(entry));
            BibtexWalker::walk_entry(self, entry);
        }
    }

    fn visit_field(&mut self, field: &'a BibtexField) {
        if range::contains(field.range(), self.position) {
            self.results.push(BibtexNode::Field(field));
            BibtexWalker::walk_field(self, field);
        }
    }

    fn visit_word(&mut self, word: &'a BibtexWord) {
        if range::contains(word.range(), self.position) {
            self.results.push(BibtexNode::Word(word));
        }
    }

    fn visit_command(&mut self, command: &'a BibtexCommand) {
        if range::contains(command.range(), self.position) {
            self.results.push(BibtexNode::Command(command));
        }
    }

    fn visit_quoted_content(&mut self, content: &'a BibtexQuotedContent) {
        if range::contains(content.range(), self.position) {
            self.results.push(BibtexNode::QuotedContent(content));
            BibtexWalker::walk_quoted_content(self, content);
        }
    }

    fn visit_braced_content(&mut self, content: &'a BibtexBracedContent) {
        if range::contains(content.range(), self.position) {
            self.results.push(BibtexNode::BracedContent(content));
            BibtexWalker::walk_braced_content(self, content);
        }
    }

    fn visit_concat(&mut self, concat: &'a BibtexConcat) {
        if range::contains(concat.range(), self.position) {
            self.results.push(BibtexNode::Concat(concat));
            BibtexWalker::walk_concat(self, concat);
        }
    }
}
