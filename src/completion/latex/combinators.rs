use crate::feature::FeatureRequest;
use crate::range;
use crate::syntax::latex::analysis::environment::ENVIRONMENT_COMMANDS;
use crate::syntax::latex::analysis::finder::{LatexFinder, LatexNode};
use crate::syntax::latex::ast::*;
use crate::syntax::text::SyntaxNode;
use crate::workspace::SyntaxTree;
use lsp_types::{CompletionItem, CompletionParams, Position};

pub struct LatexCombinators;

impl LatexCombinators {
    pub async fn command<'a, E, F>(
        request: &'a FeatureRequest<CompletionParams>,
        execute: E,
    ) -> Vec<CompletionItem>
    where
        E: Fn(&'a LatexCommand) -> F,
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

    pub async fn argument<'a, E, F>(
        request: &'a FeatureRequest<CompletionParams>,
        command_names: &'a [&'a str],
        argument_index: usize,
        execute: E,
    ) -> Vec<CompletionItem>
    where
        E: Fn(&LatexCommand) -> F,
        F: std::future::Future<Output = Vec<CompletionItem>>,
    {
        let find_command = |nodes: &[LatexNode<'a>], node_index: usize| {
            if let LatexNode::Group(group) = nodes[node_index] {
                if let LatexNode::Command(command) = nodes[node_index + 1] {
                    if command_names.contains(&command.name.text())
                        && command.args.len() > argument_index
                        && command.args[argument_index] == *group
                    {
                        return Some(command);
                    }
                }
            }
            None
        };

        let find_non_empty_command = |nodes: &[LatexNode<'a>]| {
            if nodes.len() >= 3 {
                if let LatexNode::Text(_) = nodes[0] {
                    return find_command(nodes, 1);
                }
            }
            None
        };

        let find_empty_command = |nodes: &[LatexNode<'a>]| {
            if nodes.len() >= 2 {
                find_command(nodes, 0)
            } else {
                None
            }
        };

        if let SyntaxTree::Latex(tree) = &request.document.tree {
            let mut finder = LatexFinder::new(request.params.position);
            finder.visit_root(&tree.root);
            let mut nodes = finder.results;
            nodes.reverse();

            let command = find_non_empty_command(&nodes).or_else(|| find_empty_command(&nodes));

            if let Some(command) = command {
                if range::contains_exclusive(
                    command.args[argument_index].range(),
                    request.params.position,
                ) {
                    return await!(execute(command));
                }
            }
        }
        Vec::new()
    }

    pub async fn environment<E, F>(
        request: &FeatureRequest<CompletionParams>,
        execute: E,
    ) -> Vec<CompletionItem>
    where
        E: Fn(&LatexCommand) -> F,
        F: std::future::Future<Output = Vec<CompletionItem>>,
    {
        await!(Self::argument(&request, &ENVIRONMENT_COMMANDS, 0, execute))
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
