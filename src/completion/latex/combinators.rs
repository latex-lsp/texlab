use crate::data::language::language_data;
use crate::feature::FeatureRequest;
use crate::syntax::latex::*;
use crate::syntax::SyntaxTree;
use lsp_types::{CompletionItem, CompletionParams};
use std::sync::Arc;

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

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct ArgumentLocation<'a> {
    pub name: &'a str,
    pub index: usize,
}

impl<'a> ArgumentLocation<'a> {
    pub fn new(name: &'a str, index: usize) -> Self {
        Self { name, index }
    }
}

pub async fn argument<'a, I, E, F>(
    request: &'a FeatureRequest<CompletionParams>,
    mut locations: I,
    execute: E,
) -> Vec<Arc<CompletionItem>>
where
    I: Iterator<Item = ArgumentLocation<'a>>,
    E: FnOnce(Arc<LatexCommand>) -> F,
    F: std::future::Future<Output = Vec<Arc<CompletionItem>>>,
{
    let mut find_command = |nodes: &[LatexNode], node_index: usize| {
        if let LatexNode::Group(group) = &nodes[node_index] {
            if let LatexNode::Command(command) = nodes[node_index + 1].clone() {
                for location in locations.by_ref() {
                    if command.name.text() == location.name
                        && command.args.len() > location.index
                        && &command.args[location.index] == group
                    {
                        return Some((command, location.index));
                    }
                }
            }
        }
        None
    };

    if let SyntaxTree::Latex(tree) = &request.document().tree {
        let mut nodes = tree.find(request.params.position);
        nodes.reverse();

        let result1 = {
            if nodes.len() >= 3 {
                if let LatexNode::Text(_) = nodes[0] {
                    find_command(&nodes, 1)
                } else {
                    None
                }
            } else {
                None
            }
        };

        let result2 = {
            if nodes.len() >= 2 {
                find_command(&nodes, 0)
            } else {
                None
            }
        };

        if let Some((command, index)) = result1.or(result2) {
            if command.args[index]
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
    let locations = language_data()
        .environment_commands
        .iter()
        .map(|cmd| ArgumentLocation::new(&cmd.name, cmd.index));
    argument(request, locations, execute).await
}
