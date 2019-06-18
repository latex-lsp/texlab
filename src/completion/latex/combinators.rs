use crate::feature::FeatureRequest;
use crate::syntax::latex::*;
use crate::syntax::SyntaxTree;
use lsp_types::{CompletionItem, CompletionParams};
use std::sync::Arc;

pub struct LatexCombinators;

impl LatexCombinators {
    pub async fn command<E, F>(
        request: &FeatureRequest<CompletionParams>,
        execute: E,
    ) -> Vec<Arc<CompletionItem>>
    where
        E: FnOnce(Arc<LatexCommand>) -> F,
        F: std::future::Future<Output = Vec<Arc<CompletionItem>>>,
    {
        if let SyntaxTree::Latex(tree) = &request.document().tree {
            if let Some(command) = tree.find_command(request.params.position) {
                return execute(command).await;
            }
        }
        Vec::new()
    }

    pub async fn argument<'a, E, F>(
        request: &'a FeatureRequest<CompletionParams>,
        command_names: &'a [&'a str],
        argument_index: usize,
        execute: E,
    ) -> Vec<Arc<CompletionItem>>
    where
        E: FnOnce(Arc<LatexCommand>) -> F,
        F: std::future::Future<Output = Vec<Arc<CompletionItem>>>,
    {
        let find_command = |nodes: &[LatexNode], node_index: usize| {
            if let LatexNode::Group(group) = &nodes[node_index] {
                if let LatexNode::Command(command) = nodes[node_index + 1].clone() {
                    if command_names.contains(&command.name.text())
                        && command.args.len() > argument_index
                        && &command.args[argument_index] == group
                    {
                        return Some(command);
                    }
                }
            }
            None
        };

        let find_non_empty_command = |nodes: &[LatexNode]| {
            if nodes.len() >= 3 {
                if let LatexNode::Text(_) = nodes[0] {
                    return find_command(nodes, 1);
                }
            }
            None
        };

        let find_empty_command = |nodes: &[LatexNode]| {
            if nodes.len() >= 2 {
                find_command(nodes, 0)
            } else {
                None
            }
        };

        if let SyntaxTree::Latex(tree) = &request.document().tree {
            let mut nodes = tree.find(request.params.position);
            nodes.reverse();

            let command = find_non_empty_command(&nodes).or_else(|| find_empty_command(&nodes));

            if let Some(command) = command {
                if command.args[argument_index]
                    .range
                    .contains_exclusive(request.params.position)
                {
                    return execute(Arc::clone(&command)).await;
                }
            }
        }
        Vec::new()
    }

    pub async fn environment<E, F>(
        request: &FeatureRequest<CompletionParams>,
        execute: E,
    ) -> Vec<Arc<CompletionItem>>
    where
        E: FnOnce(Arc<LatexCommand>) -> F,
        F: std::future::Future<Output = Vec<Arc<CompletionItem>>>,
    {
        Self::argument(&request, &ENVIRONMENT_COMMANDS, 0, execute).await
    }
}
