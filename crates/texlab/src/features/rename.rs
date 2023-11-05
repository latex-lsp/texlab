use std::collections::HashMap;

use base_db::{FeatureParams, Workspace};
use rename::RenameParams;

use crate::util::line_index_ext::LineIndexExt;

pub fn prepare_rename_all(
    workspace: &Workspace,
    params: &lsp_types::TextDocumentPositionParams,
) -> Option<lsp_types::Range> {
    let params = create_params(workspace, params)?;
    let range = rename::prepare_rename(&params)?;
    params.inner.document.line_index.line_col_lsp_range(range)
}

pub fn rename_all(
    workspace: &Workspace,
    params: &lsp_types::RenameParams,
) -> Option<lsp_types::WorkspaceEdit> {
    let new_name = &params.new_name;
    let params = create_params(workspace, &params.text_document_position)?;
    let result = rename::rename(&params);

    let mut changes = HashMap::default();
    for (document, ranges) in result.changes {
        let mut edits = Vec::new();
        ranges
            .into_iter()
            .filter_map(|range| document.line_index.line_col_lsp_range(range))
            .for_each(|range| edits.push(lsp_types::TextEdit::new(range, new_name.clone())));

        changes.insert(document.uri.clone(), edits);
    }

    Some(lsp_types::WorkspaceEdit::new(changes))
}

fn create_params<'db>(
    workspace: &'db Workspace,
    params: &lsp_types::TextDocumentPositionParams,
) -> Option<RenameParams<'db>> {
    let document = workspace.lookup(&params.text_document.uri)?;
    let inner = FeatureParams::new(workspace, document);
    let offset = document.line_index.offset_lsp(params.position)?;
    Some(RenameParams { inner, offset })
}
