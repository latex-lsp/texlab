mod label;

use lsp_types::{InlayHint, InlayHintParams};

use self::label::find_label_inlay_hints;

use super::FeatureRequest;

pub fn find_inlay_hints(request: FeatureRequest<InlayHintParams>) -> Vec<InlayHint> {
    let mut hints = Vec::new();
    find_label_inlay_hints(&request, &mut hints);
    hints
}
