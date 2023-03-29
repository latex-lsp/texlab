use rowan::ast::AstNode;
use syntax::latex;

use crate::util::cursor::CursorContext;

use super::DefinitionResult;

pub(super) fn goto_definition(context: &CursorContext) -> Option<Vec<DefinitionResult>> {
    let name = context
        .cursor
        .as_tex()
        .filter(|token| token.kind() == latex::COMMAND_NAME)?;

    let origin_selection_range = name.text_range();

    for document in context.related() {
        if let Some(data) = document.parse(context.db).as_tex() {
            let root = data.root(context.db);
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
    }

    None
}
