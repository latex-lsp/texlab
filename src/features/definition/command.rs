use std::sync::Arc;

use lsp_types::GotoDefinitionParams;
use rowan::ast::AstNode;

use crate::{features::cursor::CursorContext, syntax::latex};

use super::DefinitionResult;

pub(super) fn goto_command_definition(
    context: &CursorContext<GotoDefinitionParams>,
) -> Option<Vec<DefinitionResult>> {
    let name = context
        .cursor
        .as_latex()
        .filter(|token| token.kind().is_command_name())?;

    let origin_selection_range = name.text_range();

    for document in context.request.workspace.iter() {
        if let Some(data) = document.data.as_latex() {
            let root = latex::SyntaxNode::new_root(data.green.clone());

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
                        target_uri: Arc::clone(&document.uri),
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
