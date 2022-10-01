use std::sync::Arc;

use lsp_types::GotoDefinitionParams;
use rowan::TextRange;

use crate::features::cursor::CursorContext;

use super::DefinitionResult;

pub(super) fn goto_document_definition(
    context: &CursorContext<GotoDefinitionParams>,
) -> Option<Vec<DefinitionResult>> {
    let document = context.request.main_document();
    let data = document.data().as_latex()?;

    let workspace = &context.request.workspace;
    let working_dir =
        workspace.working_dir(workspace.parent(&document).as_ref().unwrap_or(&document));

    for include in data
        .extras
        .explicit_links
        .iter()
        .filter(|link| link.stem_range.contains_inclusive(context.offset))
    {
        if let Some(target) = include
            .targets(&working_dir, &workspace.environment.resolver)
            .find_map(|uri| context.request.workspace.get(&uri))
        {
            return Some(vec![DefinitionResult {
                origin_selection_range: include.stem_range,
                target_uri: Arc::clone(target.uri()),
                target_range: TextRange::default(),
                target_selection_range: TextRange::default(),
            }]);
        }
    }

    None
}
