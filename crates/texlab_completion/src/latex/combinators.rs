use petgraph::graph::NodeIndex;
use std::future::Future;
use texlab_feature::{DocumentContent, FeatureRequest};
use texlab_protocol::{CompletionItem, CompletionParams, Position, Range, RangeExt};
use texlab_syntax::{latex, SyntaxNode, LANGUAGE_DATA};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Parameter<'a> {
    pub name: &'a str,
    pub index: usize,
}

pub async fn command<E, F>(
    req: &FeatureRequest<CompletionParams>,
    execute: E,
) -> Vec<CompletionItem>
where
    E: FnOnce(NodeIndex) -> F,
    F: Future<Output = Vec<CompletionItem>>,
{
    if let DocumentContent::Latex(table) = &req.current().content {
        if let Some(cmd) = table
            .tree
            .find_command_by_short_name_range(req.params.text_document_position.position)
        {
            return execute(cmd).await;
        }
    }
    Vec::new()
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct ArgumentContext<'a> {
    pub parameter: Parameter<'a>,
    pub node: NodeIndex,
    pub range: Range,
}

pub async fn argument<'a, I, E, F>(
    req: &'a FeatureRequest<CompletionParams>,
    mut parameters: I,
    execute: E,
) -> Vec<CompletionItem>
where
    I: Iterator<Item = Parameter<'a>>,
    E: FnOnce(ArgumentContext<'a>) -> F,
    F: Future<Output = Vec<CompletionItem>>,
{
    if let DocumentContent::Latex(table) = &req.current().content {
        let pos = req.params.text_document_position.position;
        if let Some(node) = find_command(&table, pos) {
            let cmd = table.tree.as_command(node).unwrap();
            for parameter in parameters
                .by_ref()
                .filter(|param| param.name == cmd.name.text())
            {
                if let Some(args_node) =
                    table
                        .tree
                        .extract_group(node, latex::GroupKind::Group, parameter.index)
                {
                    let args = table.tree.as_group(args_node).unwrap();
                    if args.right.is_some() && !args.range().contains_exclusive(pos) {
                        continue;
                    }

                    let range = table
                        .tree
                        .children(args_node)
                        .filter_map(|child| table.tree.as_text(child))
                        .flat_map(|text| text.words.iter())
                        .map(|word| word.range())
                        .find(|range| range.contains(pos))
                        .unwrap_or_else(|| Range::new(pos, pos));

                    let context = ArgumentContext {
                        parameter,
                        node,
                        range,
                    };
                    return execute(context).await;
                }
            }
        }
    }
    Vec::new()
}

pub async fn argument_word<'a, I, E, F>(
    req: &'a FeatureRequest<CompletionParams>,
    mut parameters: I,
    execute: E,
) -> Vec<CompletionItem>
where
    I: Iterator<Item = Parameter<'a>>,
    E: FnOnce(NodeIndex, usize) -> F,
    F: Future<Output = Vec<CompletionItem>>,
{
    if let DocumentContent::Latex(table) = &req.current().content {
        let pos = req.params.text_document_position.position;
        if let Some(node) = find_command(&table, pos) {
            let cmd = table.tree.as_command(node).unwrap();
            for parameter in parameters
                .by_ref()
                .filter(|param| param.name == cmd.name.text())
            {
                if let Some(args_node) =
                    table
                        .tree
                        .extract_group(node, latex::GroupKind::Group, parameter.index)
                {
                    let args = table.tree.as_group(args_node).unwrap();
                    if args.right.is_some() && !args.range().contains_exclusive(pos) {
                        continue;
                    }

                    if table.tree.children(args_node).next().is_some()
                        && table
                            .tree
                            .extract_word(node, latex::GroupKind::Group, parameter.index)
                            .is_none()
                    {
                        continue;
                    }

                    return execute(node, parameter.index).await;
                }
            }
        }
    }
    Vec::new()
}

pub async fn environment<'a, E, F>(
    req: &'a FeatureRequest<CompletionParams>,
    execute: E,
) -> Vec<CompletionItem>
where
    E: FnOnce(ArgumentContext<'a>) -> F,
    F: Future<Output = Vec<CompletionItem>>,
{
    let parameters = LANGUAGE_DATA
        .environment_commands
        .iter()
        .map(|cmd| Parameter {
            name: &cmd.name,
            index: cmd.index,
        });
    argument(req, parameters, execute).await
}

fn find_command(table: &latex::SymbolTable, pos: Position) -> Option<NodeIndex> {
    table
        .tree
        .find(pos)
        .into_iter()
        .rev()
        .find(|node| table.tree.as_command(*node).is_some())
}
