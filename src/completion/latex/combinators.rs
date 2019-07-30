use crate::feature::FeatureRequest;
use lsp_types::*;
use std::future::Future;
use std::sync::Arc;
use texlab_syntax::*;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Parameter<'a> {
    pub name: &'a str,
    pub index: usize,
}

impl<'a> Parameter<'a> {
    pub fn new(name: &'a str, index: usize) -> Self {
        Self { name, index }
    }
}

pub async fn command<E, F>(
    request: &FeatureRequest<CompletionParams>,
    execute: E,
) -> Vec<CompletionItem>
where
    E: FnOnce(Arc<LatexCommand>) -> F,
    F: Future<Output = Vec<CompletionItem>>,
{
    if let SyntaxTree::Latex(tree) = &request.document().tree {
        if let Some(command) = tree.find_command_by_name(request.params.position) {
            return execute(command).await;
        }
    }
    Vec::new()
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ArgumentContext<'a> {
    pub parameter: Parameter<'a>,
    pub command: Arc<LatexCommand>,
    pub range: Range,
}

pub async fn argument<'a, I, E, F>(
    request: &'a FeatureRequest<CompletionParams>,
    mut parameters: I,
    execute: E,
) -> Vec<CompletionItem>
where
    I: Iterator<Item = Parameter<'a>>,
    E: FnOnce(ArgumentContext<'a>) -> F,
    F: Future<Output = Vec<CompletionItem>>,
{
    if let SyntaxTree::Latex(tree) = &request.document().tree {
        let position = request.params.position;
        if let Some(command) = find_command(tree, position) {
            for parameter in parameters.by_ref() {
                if command.name.text() != parameter.name {
                    continue;
                }

                if let Some(args) = command.args.get(parameter.index) {
                    if args.right.is_some() && !args.range().contains_exclusive(position) {
                        continue;
                    }

                    let mut range = None;
                    for child in &args.children {
                        if let LatexContent::Text(text) = &child {
                            for word in &text.words {
                                if word.range().contains(position) {
                                    range = Some(word.range());
                                    break;
                                }
                            }
                        }
                    }
                    let text_range = range.unwrap_or_else(|| Range::new(position, position));
                    let context = ArgumentContext {
                        parameter,
                        command: Arc::clone(&command),
                        range: text_range,
                    };
                    return execute(context).await;
                }
            }
        }
    }
    Vec::new()
}

pub async fn argument_word<'a, I, E, F>(
    request: &'a FeatureRequest<CompletionParams>,
    mut parameters: I,
    execute: E,
) -> Vec<CompletionItem>
where
    I: Iterator<Item = Parameter<'a>>,
    E: FnOnce(Arc<LatexCommand>, usize) -> F,
    F: Future<Output = Vec<CompletionItem>>,
{
    if let SyntaxTree::Latex(tree) = &request.document().tree {
        let position = request.params.position;
        if let Some(command) = find_command(tree, position) {
            for parameter in parameters.by_ref() {
                if command.name.text() != parameter.name {
                    continue;
                }

                if let Some(args) = command.args.get(parameter.index) {
                    if args.right.is_some() && !args.range().contains_exclusive(position) {
                        continue;
                    }

                    if args.children.len() != 0 && !command.has_word(parameter.index) {
                        continue;
                    }

                    return execute(Arc::clone(&command), parameter.index).await;
                }
            }
        }
    }
    Vec::new()
}

pub async fn environment<'a, E, F>(
    request: &'a FeatureRequest<CompletionParams>,
    execute: E,
) -> Vec<CompletionItem>
where
    E: FnOnce(ArgumentContext<'a>) -> F,
    F: Future<Output = Vec<CompletionItem>>,
{
    let parameters = LANGUAGE_DATA
        .environment_commands
        .iter()
        .map(|cmd| Parameter::new(&cmd.name, cmd.index));
    argument(request, parameters, execute).await
}

fn find_command(tree: &LatexSyntaxTree, position: Position) -> Option<Arc<LatexCommand>> {
    let mut nodes = tree.find(position);
    nodes.reverse();
    for node in nodes {
        if let LatexNode::Command(command) = node {
            return Some(command);
        }
    }
    None
}
