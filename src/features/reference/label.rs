use std::sync::Arc;

use lsp_types::ReferenceParams;

use crate::features::cursor::CursorContext;

use super::ReferenceResult;

pub(super) fn find_label_references(
    context: &CursorContext<ReferenceParams>,
    results: &mut Vec<ReferenceResult>,
) -> Option<()> {
    let (name_text, _) = context
        .find_label_name_key()
        .or_else(|| context.find_label_name_command())?;

    for document in context.request.workspace.iter() {
        if let Some(data) = document.data().as_latex() {
            for name in data
                .extras
                .label_names
                .iter()
                .filter(|name| name.text == name_text)
                .filter(|name| {
                    !name.is_definition || context.request.params.context.include_declaration
                })
            {
                results.push(ReferenceResult {
                    uri: Arc::clone(document.uri()),
                    range: name.range,
                });
            }
        }
    }

    Some(())
}
