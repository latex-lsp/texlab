mod include;

use std::sync::Arc;

use lsp_types::{DocumentLink, DocumentLinkParams, Url};
use rowan::TextRange;

use crate::LineIndexExt;

use self::include::find_include_links;

use super::FeatureRequest;

pub fn find_document_links(request: FeatureRequest<DocumentLinkParams>) -> Vec<DocumentLink> {
    let document = request.main_document();
    let mut results = Vec::new();
    find_include_links(&request, &mut results);
    results
        .into_iter()
        .map(|result| DocumentLink {
            range: document.line_index.line_col_lsp_range(result.range),
            target: Some(result.target.as_ref().clone()),
            tooltip: None,
            data: None,
        })
        .collect()
}

#[derive(Debug, Clone)]
struct LinkResult {
    range: TextRange,
    target: Arc<Url>,
}

#[cfg(test)]
mod tests;
