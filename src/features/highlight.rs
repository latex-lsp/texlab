mod label;

use cancellation::CancellationToken;
use lsp_types::{DocumentHighlight, DocumentHighlightParams};

use self::label::find_label_highlights;

use super::FeatureRequest;

pub fn find_document_highlights(
    request: FeatureRequest<DocumentHighlightParams>,
    token: &CancellationToken,
) -> Option<Vec<DocumentHighlight>> {
    find_label_highlights(&request, token)
}
