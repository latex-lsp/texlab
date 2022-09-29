use std::sync::Arc;

use lsp_types::GotoDefinitionParams;

use crate::{features::cursor::CursorContext, find_label_definition, render_label, syntax::latex};

use super::DefinitionResult;

pub(super) fn goto_label_definition(
    context: &CursorContext<GotoDefinitionParams>,
) -> Option<Vec<DefinitionResult>> {
    let (name_text, origin_selection_range) = context
        .find_label_name_key()
        .or_else(|| context.find_label_name_command())?;

    for document in context.request.workspace.iter() {
        if let Some(data) = document.data.as_latex() {
            let root = latex::SyntaxNode::new_root(data.green.clone());
            if let Some(definition) = find_label_definition(&root, &name_text) {
                let target_selection_range = latex::small_range(&definition.name()?.key()?);
                let target_range =
                    render_label(&context.request.workspace, &name_text, Some(definition))
                        .map(|label| label.range)
                        .unwrap_or(target_selection_range);

                return Some(vec![DefinitionResult {
                    origin_selection_range,
                    target_uri: Arc::clone(&document.uri),
                    target_range,
                    target_selection_range,
                }]);
            }
        }
    }

    None
}
