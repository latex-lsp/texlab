use base_db::{FeatureParams, Workspace};
use references::{ReferenceKind, ReferenceParams};

use crate::util::line_index_ext::LineIndexExt;

pub fn find_all(
    workspace: &Workspace,
    params: lsp_types::ReferenceParams,
) -> Option<Vec<lsp_types::Location>> {
    let uri_and_pos = params.text_document_position;
    let include_declaration = params.context.include_declaration;

    let document = workspace.lookup(&uri_and_pos.text_document.uri)?;
    let offset = document.line_index.offset_lsp(uri_and_pos.position)?;

    let feature = FeatureParams::new(workspace, document);
    let mut results = Vec::new();
    for result in references::find_all(ReferenceParams { feature, offset })
        .into_iter()
        .filter(|result| result.kind == ReferenceKind::Reference || include_declaration)
    {
        let document = result.location.document;
        let uri = document.uri.clone();
        if let Some(range) = document
            .line_index
            .line_col_lsp_range(result.location.range)
        {
            let location = lsp_types::Location::new(uri, range);
            results.push(location);
        }
    }

    Some(results)
}
