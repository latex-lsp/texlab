use crate::{
    feature::FeatureRequest,
    protocol::{CompletionParams, Position, Range, RangeExt},
    syntax::{latex, AstNodeIndex, SyntaxNode, LANGUAGE_DATA},
    workspace::DocumentContent,
};
use std::future::Future;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Parameter<'a> {
    pub name: &'a str,
    pub index: usize,
}

pub async fn command<E, F>(req: &FeatureRequest<CompletionParams>, execute: E)
where
    E: FnOnce(AstNodeIndex) -> F,
    F: Future<Output = ()>,
{
    if let DocumentContent::Latex(table) = &req.current().content {
        let pos = req.params.text_document_position.position;
        if let Some(cmd) = table.find_command_by_short_name_range(pos) {
            execute(cmd).await;
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct ArgumentContext<'a> {
    pub parameter: Parameter<'a>,
    pub node: AstNodeIndex,
    pub range: Range,
}

pub async fn argument<'a, I, E, F>(
    req: &'a FeatureRequest<CompletionParams>,
    mut parameters: I,
    execute: E,
) where
    I: Iterator<Item = Parameter<'a>>,
    E: FnOnce(ArgumentContext<'a>) -> F,
    F: Future<Output = ()>,
{
    if let DocumentContent::Latex(table) = &req.current().content {
        let pos = req.params.text_document_position.position;
        if let Some(node) = find_command(&table, pos) {
            let cmd = table.as_command(node).unwrap();
            for parameter in parameters
                .by_ref()
                .filter(|param| param.name == &cmd.name.text()[1..])
            {
                if let Some(args_node) =
                    table.extract_group(node, latex::GroupKind::Group, parameter.index)
                {
                    let args = table.as_group(args_node).unwrap();
                    if args.right.is_some() && !args.range().contains_exclusive(pos) {
                        continue;
                    }

                    let range = table
                        .children(args_node)
                        .filter_map(|child| table.as_text(child))
                        .flat_map(|text| text.words.iter())
                        .map(|word| word.range())
                        .find(|range| range.contains(pos))
                        .unwrap_or_else(|| Range::new(pos, pos));

                    let context = ArgumentContext {
                        parameter,
                        node,
                        range,
                    };
                    execute(context).await;
                    return;
                }
            }
        }
    }
}

pub async fn argument_word<'a, I, E, F>(
    req: &'a FeatureRequest<CompletionParams>,
    mut parameters: I,
    execute: E,
) where
    I: Iterator<Item = Parameter<'a>>,
    E: FnOnce(AstNodeIndex, usize) -> F,
    F: Future<Output = ()>,
{
    if let DocumentContent::Latex(table) = &req.current().content {
        let pos = req.params.text_document_position.position;
        if let Some(node) = find_command(&table, pos) {
            let cmd = table.as_command(node).unwrap();
            for parameter in parameters
                .by_ref()
                .filter(|param| param.name == &cmd.name.text()[1..])
            {
                if let Some(args_node) =
                    table.extract_group(node, latex::GroupKind::Group, parameter.index)
                {
                    let args = table.as_group(args_node).unwrap();
                    if args.right.is_some() && !args.range().contains_exclusive(pos) {
                        continue;
                    }

                    if table.children(args_node).next().is_some()
                        && table
                            .extract_word(node, latex::GroupKind::Group, parameter.index)
                            .is_none()
                    {
                        continue;
                    }

                    execute(node, parameter.index).await;
                    return;
                }
            }
        }
    }
}

pub async fn environment<'a, E, F>(req: &'a FeatureRequest<CompletionParams>, execute: E)
where
    E: FnOnce(ArgumentContext<'a>) -> F,
    F: Future<Output = ()>,
{
    let parameters = LANGUAGE_DATA
        .environment_commands
        .iter()
        .map(|cmd| Parameter {
            name: &cmd.name[1..],
            index: cmd.index,
        });
    argument(req, parameters, execute).await;
}

fn find_command(table: &latex::SymbolTable, pos: Position) -> Option<AstNodeIndex> {
    table
        .find(pos)
        .into_iter()
        .rev()
        .find(|node| table.as_command(*node).is_some())
}
