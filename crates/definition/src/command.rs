use base_db::DocumentData;
use rowan::ast::AstNode;
use syntax::latex;

use crate::DefinitionContext;

use super::DefinitionResult;

pub(super) fn goto_definition<'db>(context: &mut DefinitionContext<'db>) -> Option<()> {
    let data = context.params.document.data.as_tex()?;
    let root = data.root_node();
    let name = root
        .token_at_offset(context.params.offset)
        .find(|token| token.kind() == latex::COMMAND_NAME)?;

    let origin_selection_range = name.text_range();

    for document in &context.project.documents {
        let DocumentData::Tex(data) = &document.data else {
            continue;
        };

        let results = data
            .root_node()
            .descendants()
            .filter_map(latex::CommandDefinition::cast)
            .filter(|def| {
                def.name()
                    .and_then(|name| name.command())
                    .map_or(false, |node| node.text() == name.text())
            })
            .filter_map(|def| {
                Some(DefinitionResult {
                    origin_selection_range,
                    target: document,
                    target_range: latex::small_range(&def),
                    target_selection_range: def.name()?.command()?.text_range(),
                })
            });

        context.results.extend(results);
    }

    Some(())
}
