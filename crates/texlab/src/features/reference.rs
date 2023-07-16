use base_db::Workspace;
use references::{ReferenceKind, ReferenceParams};

use crate::util::line_index_ext::LineIndexExt;

pub fn find_all(
    workspace: &Workspace,
    uri: &lsp_types::Url,
    position: lsp_types::Position,
    context: &lsp_types::ReferenceContext,
) -> Option<Vec<lsp_types::Location>> {
    let document = workspace.lookup(uri)?;
    let offset = document.line_index.offset_lsp(position);
    let params = ReferenceParams {
        workspace,
        document,
        offset,
    };

    let mut results = Vec::new();
    for result in references::find_all(params)
        .into_iter()
        .filter(|result| result.kind == ReferenceKind::Reference || context.include_declaration)
    {
        let document = result.document;
        let uri = document.uri.clone();
        let range = document.line_index.line_col_lsp_range(result.range);
        let location = lsp_types::Location::new(uri, range);
        results.push(location);
    }

    Some(results)
}
