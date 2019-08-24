use super::ast::*;
use crate::range::RangeExt;
use crate::syntax::text::SyntaxNode;
use lsp_types::Position;
use std::sync::Arc;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum LatexNode {
    Root(Arc<LatexRoot>),
    Group(Arc<LatexGroup>),
    Command(Arc<LatexCommand>),
    Text(Arc<LatexText>),
    Comma(Arc<LatexComma>),
    Math(Arc<LatexMath>),
}

#[derive(Debug)]
pub struct LatexFinder {
    pub position: Position,
    pub results: Vec<LatexNode>,
}

impl LatexFinder {
    pub fn new(position: Position) -> Self {
        Self {
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
            self.results.push(LatexNode::Text(Arc::clone(&text)));
            LatexWalker::walk_text(self, text);
        }
    }

    fn visit_comma(&mut self, comma: Arc<LatexComma>) {
        if comma.range().contains(self.position) {
            self.results.push(LatexNode::Comma(Arc::clone(&comma)));
            LatexWalker::walk_comma(self, comma);
        }
    }

    fn visit_math(&mut self, math: Arc<LatexMath>) {
        if math.range().contains(self.position) {
            self.results.push(LatexNode::Math(Arc::clone(&math)));
            LatexWalker::walk_math(self, math);
        }
    }
}
