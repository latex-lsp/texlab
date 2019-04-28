use crate::range;
use crate::syntax::latex::ast::*;
use crate::syntax::text::SyntaxNode;
use lsp_types::Position;

pub enum LatexNode<'a> {
    Root(&'a LatexRoot),
    Group(&'a LatexGroup),
    Command(&'a LatexCommand),
    Text(&'a LatexText),
}

pub struct LatexFinder<'a> {
    pub position: Position,
    pub results: Vec<LatexNode<'a>>,
}

impl<'a> LatexFinder<'a> {
    pub fn new(position: Position) -> Self {
        LatexFinder {
            position,
            results: Vec::new(),
        }
    }
}

impl<'a> LatexVisitor<'a> for LatexFinder<'a> {
    fn visit_root(&mut self, root: &'a LatexRoot) {
        if range::contains(root.range(), self.position) {
            self.results.push(LatexNode::Root(root));
            LatexWalker::walk_root(self, root);
        }
    }

    fn visit_group(&mut self, group: &'a LatexGroup) {
        if range::contains(group.range(), self.position) {
            self.results.push(LatexNode::Group(group));
            LatexWalker::walk_group(self, group);
        }
    }

    fn visit_command(&mut self, command: &'a LatexCommand) {
        if range::contains(command.range(), self.position) {
            self.results.push(LatexNode::Command(command));
            LatexWalker::walk_command(self, command);
        }
    }

    fn visit_text(&mut self, text: &'a LatexText) {
        if range::contains(text.range(), self.position) {
            self.results.push(LatexNode::Text(text));
        }
    }
}

pub struct LatexCommandFinder<'a> {
    position: Position,
    pub result: Option<&'a LatexCommand>,
}

impl<'a> LatexCommandFinder<'a> {
    pub fn new(position: Position) -> Self {
        LatexCommandFinder {
            position,
            result: None,
        }
    }
}

impl<'a> LatexVisitor<'a> for LatexCommandFinder<'a> {
    fn visit_root(&mut self, root: &'a LatexRoot) {
        if range::contains(root.range(), self.position) {
            LatexWalker::walk_root(self, root);
        }
    }

    fn visit_group(&mut self, group: &'a LatexGroup) {
        if range::contains(group.range(), self.position) {
            LatexWalker::walk_group(self, group);
        }
    }

    fn visit_command(&mut self, command: &'a LatexCommand) {
        if range::contains(command.name.range(), self.position)
            && command.name.start().character != self.position.character
        {
            self.result = Some(command);
            return;
        }

        if range::contains(command.range(), self.position) {
            LatexWalker::walk_command(self, command);
        }
    }

    fn visit_text(&mut self, text: &'a LatexText) {}
}
