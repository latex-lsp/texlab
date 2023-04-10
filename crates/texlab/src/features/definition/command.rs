use base_db::DocumentData;
use rowan::ast::AstNode;
use syntax::latex;

use crate::util::cursor::CursorContext;

use super::DefinitionResult;

pub(super) fn goto_definition<'a>(
    context: &CursorContext<'a>,
) -> Option<Vec<DefinitionResult<'a>>> {
    let name = context
        .cursor
        .as_tex()
        .filter(|token| token.kind() == latex::COMMAND_NAME)?;

    let origin_selection_range = name.text_range();

    for document in &context.project.documents {
        let DocumentData::Tex(data) = &document.data else { continue };

        let root = data.root_node();
        if let Some(result) = root
            .descendants()
            .filter_map(latex::CommandDefinition::cast)
            .filter(|def| {
                def.name()
                    .and_then(|name| name.command())
                    .map_or(false, |node| node.text() == name.text())
            })
            .find_map(|def| {
                Some(DefinitionResult {
                    origin_selection_range,
                    target: document,
                    target_range: latex::small_range(&def),
                    target_selection_range: def.name()?.command()?.text_range(),
                })
            })
        {
            return Some(vec![result]);
        }
    }

    None
}
