use crate::syntax::latex::ast::*;
use crate::syntax::text::SyntaxNode;
use lsp_types::Position;
use std::sync::Arc;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum LatexNode {
    Root(Arc<LatexRoot>),
    Group(Arc<LatexGroup>),
    Command(Arc<LatexCommand>),
    Text(Arc<LatexText>),
}

pub struct LatexFinder {
    pub position: Position,
    pub results: Vec<LatexNode>,
}

impl LatexFinder {
    pub fn new(position: Position) -> Self {
        LatexFinder {
            position,
            results: Vec::new(),
        }
    }
}

impl LatexVisitor for LatexFinder {
    fn visit_root(&mut self, root: Arc<LatexRoot>) {
        if root.range().contains(self.position) {
            self.results.push(LatexNode::Root(Arc::clone(&root)));
            LatexWalker::walk_root(self, root);
        }
    }

    fn visit_group(&mut self, group: Arc<LatexGroup>) {
        if group.range.contains(self.position) {
            self.results.push(LatexNode::Group(Arc::clone(&group)));
            LatexWalker::walk_group(self, group);
        }
    }

    fn visit_command(&mut self, command: Arc<LatexCommand>) {
        if command.range.contains(self.position) {
            self.results.push(LatexNode::Command(Arc::clone(&command)));
            LatexWalker::walk_command(self, command);
        }
    }

    fn visit_text(&mut self, text: Arc<LatexText>) {
        if text.range.contains(self.position) {
            self.results.push(LatexNode::Text(text));
        }
    }
}
