mod entry;
mod label;
mod string;

use std::sync::Arc;

use lsp_types::{Location, ReferenceParams, Url};
use rowan::TextRange;

use crate::LineIndexExt;

use self::{
    entry::find_entry_references, label::find_label_references, string::find_string_references,
};

use super::{cursor::CursorContext, FeatureRequest};

pub fn find_all_references(request: FeatureRequest<ReferenceParams>) -> Vec<Location> {
    let mut results = Vec::new();
    let context = CursorContext::new(request);
    log::debug!("[References] Cursor: {:?}", context.cursor);
    find_label_references(&context, &mut results);
    find_entry_references(&context, &mut results);
    find_string_references(&context, &mut results);

    results
        .into_iter()
        .map(|result| Location {
            uri: result.uri.as_ref().clone(),
            range: context.request.workspace.documents_by_uri[&result.uri]
                .line_index
                .line_col_lsp_range(result.range),
        })
        .collect()
}

#[derive(Debug, Clone)]
struct ReferenceResult {
    uri: Arc<Url>,
    range: TextRange,
}

#[cfg(test)]
mod tests;
