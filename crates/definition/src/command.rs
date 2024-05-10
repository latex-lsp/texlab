use base_db::DocumentData;
use rowan::{ast::AstNode, TextRange};
use syntax::latex;

use crate::DefinitionContext;

use super::DefinitionResult;

pub(super) fn goto_definition(context: &mut DefinitionContext) -> Option<()> {
    let feature = &context.params.feature;
    let data = feature.document.data.as_tex()?;
    let root = data.root_node();
    let name = root
        .token_at_offset(context.params.offset)
        .find(|token| token.kind() == latex::COMMAND_NAME)?;

    let origin_selection_range = name.text_range();

    for document in &feature.project.documents {
        let DocumentData::Tex(data) = &document.data else {
            continue;
        };

        let results = data
            .root_node()
            .descendants()
            .filter_map(|node| {
                process_old_definition(node.clone()).or_else(|| process_new_definition(node))
            })
            .filter(|(_, command)| command.text() == name.text())
            .map(|(target_range, command)| DefinitionResult {
                origin_selection_range,
                target: document,
                target_range,
                target_selection_range: command.text_range(),
            });

        context.results.extend(results);
    }

    Some(())
}

fn process_old_definition(node: latex::SyntaxNode) -> Option<(TextRange, latex::SyntaxToken)> {
    let node = latex::OldCommandDefinition::cast(node)?;
    let name = node.name()?;
    Some((latex::small_range(&node), name))
}

fn process_new_definition(node: latex::SyntaxNode) -> Option<(TextRange, latex::SyntaxToken)> {
    let node = latex::NewCommandDefinition::cast(node)?;
    let name = node.name()?.command()?;
    Some((latex::small_range(&node), name))
}
