use std::sync::Arc;

use lsp_types::GotoDefinitionParams;
use rowan::TextRange;

use crate::features::cursor::CursorContext;

use super::DefinitionResult;

pub(super) fn goto_document_definition(
    context: &CursorContext<GotoDefinitionParams>,
) -> Option<Vec<DefinitionResult>> {
    let data = context.request.main_document().data.as_latex()?;

    for include in data
        .extras
        .explicit_links
        .iter()
        .filter(|link| link.stem_range.contains_inclusive(context.offset))
    {
        for target in &include.targets {
            if context
                .request
                .workspace
                .documents_by_uri
                .values()
                .any(|document| document.uri.as_ref() == target.as_ref())
            {
                return Some(vec![DefinitionResult {
                    origin_selection_range: include.stem_range,
                    target_uri: Arc::clone(&target),
                    target_range: TextRange::default(),
                    target_selection_range: TextRange::default(),
                }]);
            }
        }
    }

    None
}
