use base_db::Workspace;

use crate::util::{from_proto, line_index_ext::LineIndexExt, to_proto};

pub fn prepare_rename_all(
    workspace: &Workspace,
    params: lsp_types::TextDocumentPositionParams,
) -> Option<lsp_types::Range> {
    let params = from_proto::rename_params(workspace, params)?;
    let range = rename::prepare_rename(&params)?;
    params.feature.document.line_index.line_col_lsp_range(range)
}

pub fn rename_all(
    workspace: &Workspace,
    params: lsp_types::RenameParams,
) -> Option<lsp_types::WorkspaceEdit> {
    let new_name = &params.new_name;
    let params = from_proto::rename_params(workspace, params.text_document_position)?;
    let result = rename::rename(params);
    Some(to_proto::workspace_edit(result, &new_name))
}
