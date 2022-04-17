mod label;

use lsp_types::{DocumentHighlight, DocumentHighlightParams};

use self::label::find_label_highlights;

use super::{cursor::CursorContext, FeatureRequest};

pub fn find_document_highlights(
    request: FeatureRequest<DocumentHighlightParams>,
) -> Option<Vec<DocumentHighlight>> {
    let context = CursorContext::new(request);
    find_label_highlights(&context)
}
