use crate::feature::FeatureRequest;
use crate::range;
use crate::syntax::latex::analysis::command::LatexCommandAnalyzer;
use crate::syntax::latex::ast::*;
use crate::syntax::text::SyntaxNode;
use crate::workspace::SyntaxTree;
use lsp_types::{CompletionItem, CompletionParams, Position};

pub struct LatexCombinators;

impl LatexCombinators {
    pub async fn command<E, F>(
        request: &FeatureRequest<CompletionParams>,
        execute: E,
    ) -> Vec<CompletionItem>
    where
        E: Fn(&LatexCommand) -> F,
        F: std::future::Future<Output = Vec<CompletionItem>>,
    {
        if let SyntaxTree::Latex(tree) = &request.document.tree {
            let mut finder = LatexCommandFinder::new(request.params.position);
            finder.visit_root(&tree.root);
            if let Some(command) = finder.result {
                return await!(execute(command));
            }
        }
        Vec::new()
    }
}

struct LatexCommandFinder<'a> {
    position: Position,
    result: Option<&'a LatexCommand>,
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
